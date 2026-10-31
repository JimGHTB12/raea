// Landmarks and headings as an accessibility tree: header (banner), nav, main, a named section (region) and an unnamed
// one (generic), an article, a named form, aside, footer (contentinfo), h1 to h3 (heading, level). The content is
// drawn at opacity 0: P16 compares the tree, the boxes still have to agree, and the pixels are P12/P13's.
// proves: aria.role.banner, aria.role.navigation, aria.role.main, aria.role.region, aria.role.article, aria.role.form, aria.role.complementary, aria.role.contentinfo, aria.role.heading, aria.property.aria-level, aria.html-aam.el-h1-h6, aria.html-aam.el-header-ancestorbody, aria.html-aam.el-footer-ancestorbody, aria.html-aam.el-aside, aria.html-aam.el-article, aria.html-aam.el-form
export const steps = [];
const box = "block h-6 w-64";
export default function Case() {
  return (
    <div data-testid="root" className="h-full w-full bg-white p-4">
      <div className="opacity-0">
        <header data-testid="header" className={box}>Site</header>
        <nav data-testid="nav" aria-label="Primary" className={box}>Links</nav>
        <main data-testid="main" className="block w-64">
          <h1 data-testid="h1" className={box}>Title</h1>
          <section data-testid="region" aria-label="Intro" className="block w-64">
            <h2 data-testid="h2" className={box}>Intro</h2>
          </section>
          <article data-testid="article" className="block w-64">
            <h3 data-testid="h3" className={box}>Post</h3>
          </article>
          <form data-testid="form" aria-label="Search" className={box} />
          <section data-testid="section" className={box}>no name</section>
          <div role="heading" aria-level={4} data-testid="h4" className={box}>Four</div>
        </main>
        <aside data-testid="aside" className={box}>Aside</aside>
        <footer data-testid="footer" className={box}>Foot</footer>
      </div>
    </div>
  );
}
