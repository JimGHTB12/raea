//! K kernels on the GPU: compiled to WGSL (wgsl.rs) and run on a wgpu device, the shared one or their own. The CPU
//! interpreter of the same text (raea-core-k) is the reference they match bit for bit.

use std::collections::HashMap;

use raea_core_k::Kernel;

mod wgsl;

pub use wgsl::wgsl_tex;

pub struct Gpu {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub adapter: String,
}

pub struct Compiled {
    pub pipeline: wgpu::ComputePipeline,
    buffers: Vec<String>,
    outputs: Vec<String>,
    textures: usize,
    pub wgsl: String,
}

/// Seconds spent in each phase of a GPU run.
#[derive(Debug, Default, Clone)]
pub struct Times {
    pub upload: f64,
    pub compute: f64,
    pub readback: f64,
}

impl Times {
    pub fn json(&self) -> serde_json::Value {
        serde_json::json!({"upload_ms": self.upload * 1e3, "compute_ms": self.compute * 1e3, "readback_ms": self.readback * 1e3})
    }
}

impl Gpu {
    /// Whether this machine has a hardware GPU adapter (a software one, such as lavapipe, doesn't count: the CPU render
    /// is the CPU route; docs/cpu.feature C1/C3). Asked once per process.
    pub fn available() -> bool {
        static A: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
        *A.get_or_init(|| {
            let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());
            pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions { power_preference: wgpu::PowerPreference::HighPerformance, ..Default::default() }))
                .is_ok_and(|a| a.get_info().device_type != wgpu::DeviceType::Cpu)
        })
    }

    pub fn new() -> Result<Gpu, String> {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions { power_preference: wgpu::PowerPreference::HighPerformance, ..Default::default() }))
            .map_err(|e| format!("no GPU adapter: {e}"))?;
        let info = adapter.get_info();
        // the adapter's own limits: always grantable (the defaults ask for 16 inter-stage variables, more than an A15's
        // 15), and they carry its big storage buffers
        let limits = adapter.limits();
        // timestamps, where the adapter has them (stage timing on the GPU; nothing else depends on them)
        let required_features = adapter.features() & (wgpu::Features::TIMESTAMP_QUERY | wgpu::Features::TIMESTAMP_QUERY_INSIDE_ENCODERS);
        let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor { required_limits: limits, required_features, ..Default::default() })).map_err(|e| format!("device: {e}"))?;
        Ok(Gpu { device, queue, adapter: format!("{} ({:?})", info.name, info.backend) })
    }

    pub fn compile(&self, k: &Kernel, buffers: &[&str], scalars: &HashMap<&str, i32>, outputs: &[&str]) -> Result<Compiled, String> {
        self.compile_tex(k, buffers, &[], scalars, outputs)
    }

    pub fn compile_tex(&self, k: &Kernel, buffers: &[&str], textures: &[&str], scalars: &HashMap<&str, i32>, outputs: &[&str]) -> Result<Compiled, String> {
        let src = wgsl_tex(k, buffers, textures, scalars, outputs)?;
        let module = self.device.create_shader_module(wgpu::ShaderModuleDescriptor { label: Some("kgpu k"), source: wgpu::ShaderSource::Wgsl(src.clone().into()) });
        let pipeline = self.device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("kgpu k"),
            layout: None,
            module: &module,
            entry_point: Some("main"),
            compilation_options: Default::default(),
            cache: None,
        });
        Ok(Compiled { pipeline, buffers: buffers.iter().map(|s| s.to_string()).collect(), outputs: outputs.iter().map(|s| s.to_string()).collect(), textures: textures.len(), wgsl: src })
    }

    /// Run over elements 0..n; returns the outputs and how long each phase took.
    pub fn run(&self, c: &Compiled, inputs: &[&[u32]], n: usize) -> Result<(Vec<Vec<u32>>, Times), String> {
        self.run_tex(c, inputs, &[], n)
    }

    /// A storage buffer on this device (`bytes` rounded up to a word), optionally filled.
    pub fn buffer(&self, bytes: usize, data: Option<&[u8]>) -> wgpu::Buffer {
        let size = (bytes.max(4) as u64).div_ceil(4) * 4;
        let b = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });
        if let Some(d) = data {
            // straight from the caller's bytes when they fill whole words (NV12 frames do); padded otherwise
            if d.len() as u64 == size {
                self.queue.write_buffer(&b, 0, d);
            } else {
                let mut d = d.to_vec();
                d.resize(size as usize, 0);
                self.queue.write_buffer(&b, 0, &d);
            }
        }
        b
    }

    /// Run over resident buffers: nothing uploaded, nothing read back (the outputs stay on the GPU). Waits for it
    /// when `wait`, so the caller can time the GPU work.
    pub fn dispatch(&self, c: &Compiled, inputs: &[&wgpu::Buffer], textures: &[&wgpu::TextureView], outputs: &[&wgpu::Buffer], n: usize, wait: bool) {
        self.dispatch_ts(c, inputs, textures, outputs, n, wait, None)
    }

    /// As `dispatch`, with the pass's start and end written to queries `i` and `i + 1` of a timestamp query set.
    #[allow(clippy::too_many_arguments)]
    pub fn dispatch_ts(&self, c: &Compiled, inputs: &[&wgpu::Buffer], textures: &[&wgpu::TextureView], outputs: &[&wgpu::Buffer], n: usize, wait: bool, ts: Option<(&wgpu::QuerySet, u32)>) {
        use wgpu::util::DeviceExt;
        let groups = n.div_ceil(256) as u32;
        let gx = groups.min(65535).max(1);
        let gy = groups.div_ceil(gx).max(1);
        let shape = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&[n as u32, gx * 256]),
            usage: wgpu::BufferUsages::UNIFORM,
        });
        let mut entries: Vec<wgpu::BindGroupEntry> = Vec::new();
        for (b, buf) in inputs.iter().enumerate() {
            entries.push(wgpu::BindGroupEntry { binding: b as u32, resource: buf.as_entire_binding() });
        }
        for (t, view) in textures.iter().enumerate() {
            entries.push(wgpu::BindGroupEntry { binding: (inputs.len() + t) as u32, resource: wgpu::BindingResource::TextureView(view) });
        }
        let base = inputs.len() + textures.len();
        for (o, buf) in outputs.iter().enumerate() {
            entries.push(wgpu::BindGroupEntry { binding: (base + o) as u32, resource: buf.as_entire_binding() });
        }
        entries.push(wgpu::BindGroupEntry { binding: (base + outputs.len()) as u32, resource: shape.as_entire_binding() });
        let bind = self.device.create_bind_group(&wgpu::BindGroupDescriptor { label: None, layout: &c.pipeline.get_bind_group_layout(0), entries: &entries });
        let mut enc = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("kgpu run") });
        {
            let timestamp_writes = ts.map(|(q, i)| wgpu::ComputePassTimestampWrites { query_set: q, beginning_of_pass_write_index: Some(i), end_of_pass_write_index: Some(i + 1) });
            let mut pass = enc.begin_compute_pass(&wgpu::ComputePassDescriptor { label: None, timestamp_writes });
            pass.set_pipeline(&c.pipeline);
            pass.set_bind_group(0, &bind, &[]);
            pass.dispatch_workgroups(gx, gy, 1);
        }
        self.queue.submit([enc.finish()]);
        raea_gpu_device::submitted(&self.queue, "kgpu", "kgpu run");
        if wait {
            let _ = self.device.poll(wgpu::PollType::wait_indefinitely());
        }
    }

    /// The first `bytes` of a buffer, read back.
    pub fn read(&self, buf: &wgpu::Buffer, bytes: usize) -> Vec<u8> {
        let size = (bytes as u64).div_ceil(4) * 4;
        let staging = self.device.create_buffer(&wgpu::BufferDescriptor { label: None, size, usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST, mapped_at_creation: false });
        let mut enc = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("kgpu run") });
        enc.copy_buffer_to_buffer(buf, 0, &staging, 0, size);
        self.queue.submit([enc.finish()]);
        raea_gpu_device::submitted(&self.queue, "kgpu", "kgpu run");
        staging.slice(..).map_async(wgpu::MapMode::Read, |_| {});
        let _ = self.device.poll(wgpu::PollType::wait_indefinitely());
        let v = staging.slice(..).get_mapped_range().expect("a mapped buffer")[..bytes].to_vec();
        v
    }

    /// As `run`, with texture views bound after the buffers.
    pub fn run_tex(&self, c: &Compiled, inputs: &[&[u32]], textures: &[&wgpu::TextureView], n: usize) -> Result<(Vec<Vec<u32>>, Times), String> {
        assert_eq!(textures.len(), c.textures);
        use wgpu::util::DeviceExt;
        let mut t = Times::default();
        let t0 = std::time::Instant::now();
        let ins: Vec<wgpu::Buffer> = inputs
            .iter()
            .map(|d| {
                let data: &[u32] = if d.is_empty() { &[0] } else { d };
                self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor { label: None, contents: bytemuck::cast_slice(data), usage: wgpu::BufferUsages::STORAGE })
            })
            .collect();
        let size = (n.max(1) * 4) as u64;
        let outs: Vec<wgpu::Buffer> = c
            .outputs
            .iter()
            .map(|_| self.device.create_buffer(&wgpu::BufferDescriptor { label: None, size, usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC, mapped_at_creation: false }))
            .collect();
        let groups = n.div_ceil(256) as u32;
        let gx = groups.min(65535);
        let gy = groups.div_ceil(gx);
        let shape = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&[n as u32, gx * 256]),
            usage: wgpu::BufferUsages::UNIFORM,
        });
        let layout = c.pipeline.get_bind_group_layout(0);
        let mut entries: Vec<wgpu::BindGroupEntry> = Vec::new();
        for (b, buf) in ins.iter().enumerate() {
            entries.push(wgpu::BindGroupEntry { binding: b as u32, resource: buf.as_entire_binding() });
        }
        for (t, view) in textures.iter().enumerate() {
            entries.push(wgpu::BindGroupEntry { binding: (ins.len() + t) as u32, resource: wgpu::BindingResource::TextureView(view) });
        }
        let base = ins.len() + textures.len();
        for (o, buf) in outs.iter().enumerate() {
            entries.push(wgpu::BindGroupEntry { binding: (base + o) as u32, resource: buf.as_entire_binding() });
        }
        entries.push(wgpu::BindGroupEntry { binding: (base + outs.len()) as u32, resource: shape.as_entire_binding() });
        let bind = self.device.create_bind_group(&wgpu::BindGroupDescriptor { label: None, layout: &layout, entries: &entries });
        self.queue.submit([]);
        self.device.poll(wgpu::PollType::wait_indefinitely()).map_err(|e| format!("poll: {e}"))?;
        t.upload = t0.elapsed().as_secs_f64();

        let t1 = std::time::Instant::now();
        let mut enc = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("kgpu run") });
        {
            let mut pass = enc.begin_compute_pass(&Default::default());
            pass.set_pipeline(&c.pipeline);
            pass.set_bind_group(0, &bind, &[]);
            pass.dispatch_workgroups(gx, gy, 1);
        }
        self.queue.submit([enc.finish()]);
        raea_gpu_device::submitted(&self.queue, "kgpu", "kgpu run");
        self.device.poll(wgpu::PollType::wait_indefinitely()).map_err(|e| format!("poll: {e}"))?;
        t.compute = t1.elapsed().as_secs_f64();

        let t2 = std::time::Instant::now();
        let staging: Vec<wgpu::Buffer> = outs
            .iter()
            .map(|_| self.device.create_buffer(&wgpu::BufferDescriptor { label: None, size, usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST, mapped_at_creation: false }))
            .collect();
        let mut enc = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("kgpu run") });
        for (o, s) in outs.iter().zip(&staging) {
            enc.copy_buffer_to_buffer(o, 0, s, 0, size);
        }
        self.queue.submit([enc.finish()]);
        raea_gpu_device::submitted(&self.queue, "kgpu", "kgpu run");
        for s in &staging {
            s.slice(..).map_async(wgpu::MapMode::Read, |_| {});
        }
        self.device.poll(wgpu::PollType::wait_indefinitely()).map_err(|e| format!("poll: {e}"))?;
        let mut result = Vec::new();
        for s in &staging {
            let v: Vec<u32> = bytemuck::cast_slice(&s.slice(..).get_mapped_range().expect("a mapped buffer")).to_vec();
            result.push(v[..n].to_vec());
        }
        t.readback = t2.elapsed().as_secs_f64();
        let _ = &c.buffers;
        Ok((result, t))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// look_ham.k (every pair of frames' Hamming distance: sum, pop, xor and word over a buffer) on the GPU equals the
    /// interpreter (skipped without a GPU).
    #[test]
    fn gpu_equals_interpreter() {
        if !Gpu::available() {
            return;
        }
        let gpu = Gpu::new().unwrap();
        let frames = 37usize;
        let mut s = 0x2545_f491_4f6c_dd1du64;
        let words: Vec<u32> = (0..frames * 64)
            .map(|_| {
                s ^= s << 13;
                s ^= s >> 7;
                s ^= s << 17;
                s as u32
            })
            .collect();
        let scalars: HashMap<&str, i32> = [("n", frames as i32)].into();
        let k = raea_core_k::parse(include_str!("../../../core/k/kernels/look_ham.k")).unwrap();
        let cpu = raea_core_k::Cpu { buffers: [("H", &words[..])].into(), scalars: scalars.clone() };
        let want = cpu.run(&k, frames * frames, &["d"]).unwrap().remove(0);
        let c = gpu.compile(&k, &["H"], &scalars, &["d"]).unwrap();
        let got = gpu.run(&c, &[&words], frames * frames).unwrap().0.remove(0);
        assert_eq!(got, want);
    }

    /// nv12.k packs four bytes with shl, the top one into the sign bit: the WGSL shifts as u32 (a signed shift there
    /// striped bright video on a Windows on ARM GPU, #82), and every byte value packs as the interpreter packs it.
    #[test]
    fn shl_into_the_sign_bit_is_unsigned() {
        let k = raea_core_k::parse("b:and[word[F;i];255]\nw:or[or[b;shl[b;8]];or[shl[b;16];shl[b;24]]]").unwrap();
        let src = wgsl_tex(&k, &["F"], &[], &HashMap::new(), &["w"]).unwrap();
        assert!(src.contains("bitcast<i32>(u32(k_b) << (u32((24)) & 31u))"), "{src}");
        if !Gpu::available() {
            return;
        }
        let gpu = Gpu::new().unwrap();
        let words: Vec<u32> = (0..256).collect();
        let cpu = raea_core_k::Cpu { buffers: [("F", &words[..])].into(), scalars: HashMap::new() };
        let want = cpu.run(&k, 256, &["w"]).unwrap().remove(0);
        assert_eq!(want[182], 0xb6b6_b6b6);
        let c = gpu.compile(&k, &["F"], &HashMap::new(), &["w"]).unwrap();
        assert_eq!(gpu.run(&c, &[&words], 256).unwrap().0.remove(0), want);
    }
}
