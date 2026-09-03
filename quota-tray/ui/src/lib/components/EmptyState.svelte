<script lang="ts">
  interface Props {
    title: string;
    body: string;
    href?: string;
    cta?: string;
  }
  let {
    title,
    body,
    href = "settings.html",
    cta = "Open Settings",
  }: Props = $props();
</script>

<section class="empty" aria-live="polite">
  <div class="art" aria-hidden="true">
    <div class="orb"></div>
    <div class="ring"></div>
    <div class="spark s1"></div>
    <div class="spark s2"></div>
  </div>
  <h2>{title}</h2>
  <p>{body}</p>
  <a href={href}>{cta}</a>
</section>

<style>
  .empty {
    margin: 0.25rem 1rem 0.75rem;
    padding: 1.35rem 1.1rem 1.2rem;
    text-align: center;
    background: var(--sheet);
    border: 1px solid var(--cream-line);
    border-radius: var(--radius);
    box-shadow: var(--shadow-soft);
    animation: rise-in 480ms var(--ease) both;
  }
  .art {
    position: relative;
    width: 88px;
    height: 88px;
    margin: 0 auto 0.85rem;
  }
  .orb {
    position: absolute;
    inset: 18px;
    border-radius: 50%;
    background: linear-gradient(160deg, var(--terracotta-soft), var(--terracotta));
    opacity: 0.9;
  }
  .ring {
    position: absolute;
    inset: 4px;
    border-radius: 50%;
    border: 2px dashed color-mix(in srgb, var(--terracotta) 45%, var(--cream-line));
    animation: spin 14s linear infinite;
  }
  .spark {
    position: absolute;
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: var(--cream-deep);
    box-shadow: 0 0 0 2px color-mix(in srgb, var(--terracotta) 25%, white);
  }
  .s1 {
    top: 8px;
    right: 14px;
    background: var(--ok);
  }
  .s2 {
    bottom: 12px;
    left: 10px;
    background: var(--chip-copilot);
  }
  h2 {
    margin: 0;
    font-family: var(--font);
    font-size: 1.15rem;
    letter-spacing: -0.02em;
  }
  p {
    margin: 0.4rem auto 0.9rem;
    max-width: 26ch;
    color: var(--ink-muted);
    font-size: 0.88rem;
    line-height: 1.4;
  }
  a {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    padding: 0.55rem 1.1rem;
    border-radius: 999px;
    background: linear-gradient(180deg, var(--terracotta-soft), var(--terracotta));
    color: #fff;
    font-weight: 700;
    font-size: 0.88rem;
    text-decoration: none;
    box-shadow: 0 8px 16px color-mix(in srgb, var(--terracotta) 25%, transparent);
  }
  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }
  @media (prefers-reduced-motion: reduce) {
    .ring {
      animation: none;
    }
  }
</style>
