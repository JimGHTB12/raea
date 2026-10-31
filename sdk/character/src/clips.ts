// One motion database per body (docs/clips.feature): every retargeted capture's frames in one binary file, each
// with its quantised features and state, ground frame, contacts, crouch, speed, tag bits and an HDC hypervector, and
// a library loaded from it that perform matches over with a search backend: the kernel's integer scoring, HDC, or a
// cheap CPU index.
//
//   bun tools/library.ts db SKELETON.json LIB          builds LIB/motion.db
//   const lib = dbLibraryAt(skel, LIB, { actions: [...] });      (clips-bun.ts: the file and clips read from disk)
//   lib.search = kernelSearch();                        or hdcSearch(), coarseSearch(), ...
import { type TagCond } from "./matching";
export { TAGS, TAG_INDEX, tagsOf, QF, NAN16, FLAG, HW, HLEVELS, HSTEP, FS, SS, writeDb, parseDb, MF_SRC, MF_SIGN, MS_SRC, MS_SIGN, qround, hbit, hlevel, hdcEncode, popcount, buildDb, dbLibrary, fullHdc, QW, MODE, SCORE_MAX, SCORE_CAP, BLOCK, kernelQuery, kernelScore, blockMins, topK, topKFast, blockFrames, kernelSearch, queryHdc, hdcSearch, coarseSearch, ivfSearch, recording, exactCost, rerank } from "./clips/all";
export type { Tag, DbClip, DbHeader, MotionDb, BuildClip, ClipSource, DbLibraryOptions, DbLibrary, KernelQuery } from "./clips/all";

export type { TagCond };
