<script lang="ts">
  import type { LandingCopy, LandingTemplate } from '$lib/i18n';

  export let copy: LandingCopy['templates'];
  export let templates: LandingTemplate[];
</script>

<section id="templates" class="templates section-frame" aria-labelledby="templates-title">
  <div class="section-heading">
    <div>
      <h2 id="templates-title">{copy.title}</h2>
    </div>
    <p>{copy.description}</p>
  </div>
  <div class="template-grid">
    {#each templates as template}
      <article class="template-card">
        <div class="template-image-wrap">
          <img
            src={template.image}
            alt={template.imageAlt}
            width="612"
            height="792"
            loading="eager"
            fetchpriority="high"
            decoding="async" />
        </div>
        <div class="template-card-copy">
          <div>
            <h3>{template.name}</h3>
            <p>{template.description}</p>
          </div>
        </div>
      </article>
    {/each}
  </div>
</section>

<style>
  .section-heading {
    display: grid;
    grid-template-columns: 1fr 0.66fr;
    align-items: end;
    gap: 48px;
    padding-left: 36px;
  }

  .section-heading p:last-child {
    max-width: 38ch;
    margin: 0 0 4px;
    color: var(--muted-ink);
    font-size: 14px;
    line-height: 1.6;
  }

  .templates {
    padding-top: 76px;
    padding-bottom: 132px;
  }

  h2 {
    font-size: clamp(42px, 5vw, 68px);
    font-weight: 700;
    line-height: 0.97;
  }

  .template-grid {
    display: grid;
    grid-template-columns: repeat(3, minmax(0, 1fr));
    gap: 18px;
    margin-top: 52px;
  }

  .template-card {
    display: grid;
    gap: 16px;
    min-width: 0;
  }

  .template-image-wrap {
    position: relative;
    display: grid;
    min-height: 360px;
    place-items: center;
    overflow: hidden;
    border: 1px solid var(--rule-strong);
    padding: 22px;
    background: #e1e7ed;
    transition:
      border-color 180ms ease,
      transform 180ms ease,
      box-shadow 180ms ease;
  }

  .template-card:hover .template-image-wrap {
    border-color: var(--blue);
    box-shadow: 7px 7px 0 rgb(151 181 222 / 35%);
    transform: translateY(-3px);
  }

  .template-image-wrap img {
    display: block;
    width: min(100%, 248px);
    height: auto;
    border: 1px solid rgb(23 33 43 / 10%);
    box-shadow: 0 12px 18px rgb(23 33 43 / 14%);
  }

  .template-card-copy {
    justify-content: space-between;
    gap: 16px;
    border-top: 1px solid var(--rule-strong);
    padding-top: 14px;
  }

  .template-card-copy h3 {
    margin: 0 0 5px;
    font-size: 17px;
    letter-spacing: -0.025em;
  }

  .template-card-copy p {
    max-width: 28ch;
    margin: 0;
    color: var(--muted-ink);
    font-size: 13px;
    line-height: 1.5;
  }

  @media (max-width: 700px) {
    .section-heading {
      grid-template-columns: 1fr;
      gap: 28px;
      padding-left: 22px;
    }

    h2 {
      font-size: clamp(41px, 12vw, 60px);
    }

    .templates {
      padding-top: 55px;
      padding-bottom: 94px;
    }

    .template-grid {
      grid-template-columns: 1fr;
      gap: 44px;
      margin-top: 40px;
    }

    .template-image-wrap {
      min-height: 420px;
    }

    .template-image-wrap img {
      width: min(70%, 260px);
    }
  }
</style>
