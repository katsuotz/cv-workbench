<section class="final-cta section-frame" aria-labelledby="cta-title">
  <div class="cta-copy">
    <h2 id="cta-title">Build your ATS-friendly CV.</h2>
  </div>
  <a class="button button-light button-large" href="/app">
    Start building <span aria-hidden="true">→</span>
  </a>
</section>

<style>
  .final-cta {
    display: flex;
    align-items: end;
    justify-content: space-between;
    gap: 32px;
    padding: 60px 64px;
    background: var(--blue);
    color: #fff;
  }

  .final-cta h2 {
    color: #fff;
    font-size: clamp(39px, 5vw, 67px);
  }

  @media (max-width: 700px) {
    .final-cta {
      width: calc(100% - 32px);
      align-items: flex-start;
      flex-direction: column;
      padding: 42px 28px;
    }
  }
</style>
