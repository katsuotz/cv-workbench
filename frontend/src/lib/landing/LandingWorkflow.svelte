<section id="how-it-works" class="workflow section-frame" aria-labelledby="workflow-title">
  <div class="workflow-intro">
    <h2 id="workflow-title">How it works.</h2>
  </div>
  <ol class="workflow-list">
    <li>
      <div class="workflow-marker" aria-hidden="true">01</div>
      <div>
        <h3>Build an ATS-ready structure</h3>
        <p>
          Add your experience, skills, education, and achievements in clear sections built for quick
          scanning.
        </p>
      </div>
    </li>
    <li>
      <div class="workflow-marker" aria-hidden="true">02</div>
      <div>
        <h3>Review the source</h3>
        <p>
          Generate exact XeLaTeX, keep the source visible, and see compiler notes when something
          needs attention.
        </p>
      </div>
    </li>
    <li>
      <div class="workflow-marker" aria-hidden="true">03</div>
      <div>
        <h3>Download your CV</h3>
        <p>Review the rendered pages, then download the PDF and the source that produced it.</p>
      </div>
    </li>
  </ol>
</section>

<style>
  .workflow {
    display: grid;
    grid-template-columns: 0.8fr 1.2fr;
    gap: clamp(50px, 11vw, 170px);
    padding-bottom: 132px;
    padding-left: 36px;
  }

  .workflow-intro h2 {
    margin-top: 10px;
    font-size: clamp(42px, 5vw, 68px);
    font-weight: 700;
    line-height: 0.97;
  }

  .workflow-list {
    display: grid;
    gap: 0;
    margin: 0;
    padding: 0;
    list-style: none;
  }

  .workflow-list li {
    display: grid;
    grid-template-columns: 52px minmax(0, 1fr);
    gap: 21px;
    padding: 0 0 27px;
  }

  .workflow-list li + li {
    padding-top: 27px;
    border-top: 1px solid var(--rule-strong);
  }

  .workflow-marker {
    color: var(--blue);
    line-height: 1.6;
  }

  .workflow-list h3 {
    margin: 0 0 5px;
    font-size: 19px;
  }

  .workflow-list p {
    max-width: 49ch;
    margin: 0;
    color: var(--muted-ink);
    font-size: 13px;
    line-height: 1.5;
  }

  @media (max-width: 940px) {
    .workflow {
      gap: 60px;
    }
  }

  @media (max-width: 700px) {
    .workflow {
      grid-template-columns: 1fr;
      gap: 28px;
      padding-bottom: 94px;
      padding-left: 22px;
    }
  }
</style>
