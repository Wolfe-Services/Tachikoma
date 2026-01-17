<script lang="ts">
  import PageHeader from "$lib/components/common/PageHeader.svelte";
  import Icon from "$lib/components/common/Icon.svelte";
  import Spinner from "$lib/components/ui/Spinner/Spinner.svelte";
  import GlassPanel from "$lib/components/ui/GlassPanel.svelte";
  import ForgeLayout from "$lib/components/forge/ForgeLayout.svelte";
  import SessionSidebar from "$lib/components/forge/SessionSidebar.svelte";
  import SessionCreationWizard from "$lib/components/forge/SessionCreationWizard.svelte";
  import {
    forgeSessionStore,
    activeSession,
    sessions,
    sessionLoading,
    sessionError,
  } from "$lib/stores/forgeSession";
  import { onMount } from "svelte";

  let showingWizard = false;
  let editingSessionId: string | null = null;
  let sacMode = false;

  // AI Participants for the Think Tank visualization
  // These map to the actual configured backends from tachikoma-forge crate
  // Roles: Architect (drafts), Analyst (critiques), Synthesizer (merges), Arbiter (resolves conflicts)
  const aiModels = [
    {
      id: "claude",
      name: "CLAUDE",
      role: "ARCHITECT",
      color: "#cc785c",
      tachiIcon: "/icons/Iconfactory-Ghost-In-The-Shell-Fuchikoma-Red.32.png",
      provider: "Anthropic",
    },
    {
      id: "gpt4",
      name: "GPT-4",
      role: "ANALYST",
      color: "#74aa9c",
      tachiIcon: "/icons/Iconfactory-Ghost-In-The-Shell-Fuchikoma-Gray.32.png",
      provider: "OpenAI",
    },
    {
      id: "gemini",
      name: "GEMINI",
      role: "SYNTHESIZER",
      color: "#8b5cf6",
      tachiIcon:
        "/icons/Iconfactory-Ghost-In-The-Shell-Fuchikoma-Purple.32.png",
      provider: "Google",
    },
    {
      id: "ollama",
      name: "OLLAMA",
      role: "LOCAL",
      color: "#4ecdc4",
      tachiIcon:
        "/icons/Iconfactory-Ghost-In-The-Shell-Fuchikoma-Yellow.32.png",
      provider: "Local",
    },
  ];

  onMount(async () => {
    // Global toggle used across the UI
    const storedSac = localStorage.getItem("tachikoma:sacMode");
    const legacy = localStorage.getItem("tachikoma:forgeLoreMode");
    if (storedSac === null && legacy !== null) {
      localStorage.setItem("tachikoma:sacMode", legacy);
    }
    sacMode = localStorage.getItem("tachikoma:sacMode") === "true";
    await forgeSessionStore.loadSessions();
  });

  function handleStartNewSession() {
    forgeSessionStore.clearActiveSession();
    editingSessionId = null;
    showingWizard = true;
  }

  function handleEditSession(event: CustomEvent<{ sessionId: string }>) {
    editingSessionId = event.detail.sessionId;
    showingWizard = true;
  }

  function handleSessionCreated(event: CustomEvent<{ sessionId: string }>) {
    forgeSessionStore.setActiveSession(event.detail.sessionId);
    showingWizard = false;
    editingSessionId = null;
  }

  function handleWizardCancelled() {
    showingWizard = false;
    editingSessionId = null;
    // If we were editing, re-activate the session
    if (editingSessionId) {
      forgeSessionStore.setActiveSession(editingSessionId);
    }
  }

  function toggleSacMode() {
    sacMode = !sacMode;
    localStorage.setItem("tachikoma:sacMode", String(sacMode));
  }
</script>

<div class="forge-page">
  <div
    class="polyglass-backdrop"
    class:enabled={sacMode}
    aria-hidden="true"
  ></div>
  <div class="manga-banner" aria-hidden="true"></div>

  <PageHeader
    title="THINK TANK"
    subtitle="Multi-model deliberation engine for spec creation and refinement"
    tag="SPEC FORGE"
    iconSrc="/icons/Iconfactory-Ghost-In-The-Shell-Fuchikoma-Blue.32.png"
    iconSize={48}
  >
    <svelte:fragment slot="actions">
      <button
        class="btn-secondary"
        type="button"
        on:click={toggleSacMode}
        aria-pressed={sacMode}
      >
        <Icon name={sacMode ? "check-circle" : "help-circle"} size={16} />
        <span>S.A.C. MODE</span>
      </button>

      {#if !showingWizard && !$activeSession}
        <button class="btn-primary" on:click={handleStartNewSession}>
          <Icon name="zap" size={16} />
          <span>NEW SESSION</span>
        </button>
      {/if}
    </svelte:fragment>
  </PageHeader>

  {#if $sessionError}
    <GlassPanel accent="red" className="error-panel">
      <div class="error-content" role="alert">
        <div class="error-left">
          <Icon name="alert-triangle" size={18} glow />
          <div class="error-text">
            <div class="error-title">Forge systems report an anomaly</div>
            <div class="error-subtitle">{$sessionError}</div>
          </div>
        </div>
        <div class="error-actions">
          <button
            class="btn-secondary"
            type="button"
            on:click={() => forgeSessionStore.clearError()}
          >
            Dismiss
          </button>
          <button
            class="btn-primary"
            type="button"
            on:click={() => forgeSessionStore.loadSessions()}
          >
            Retry
          </button>
        </div>
      </div>
    </GlassPanel>
  {/if}

  {#if showingWizard}
    <SessionCreationWizard
      editSessionId={editingSessionId}
      on:created={handleSessionCreated}
      on:cancelled={handleWizardCancelled}
    />
  {:else if $activeSession}
    <ForgeLayout
      sessionId={$activeSession.id}
      on:newSession={handleStartNewSession}
      on:editSession={handleEditSession}
    />
  {:else}
    <!-- Think Tank Welcome Screen -->
    <div class="think-tank-welcome">
      {#if $sessionLoading}
        <div class="loading-row" aria-live="polite">
          <Spinner size={18} color="var(--tachi-cyan, #4ecdc4)" />
          <span>Synchronizing sessions…</span>
        </div>
      {/if}

      <!-- AI Council Visualization -->
      <GlassPanel className="council-section" accent="cyan">
        <div class="council-ring">
          <div class="ring-glow"></div>
          <div class="ring-track"></div>

          {#each aiModels as model, i}
            <div
              class="council-node"
              style="
                --node-color: {model.color};
                --angle: {(360 / aiModels.length) * i}deg;
                --delay: {-i * 3.25}s;
              "
            >
              <div class="node-avatar">
                <img
                  src={model.tachiIcon}
                  alt={model.name}
                  class="tachi-model-icon"
                />
              </div>
              <div class="node-info">
                <span class="node-name">{model.name}</span>
                <span class="node-role">{model.role}</span>
              </div>
              <div class="node-pulse"></div>
            </div>
          {/each}

          <div class="council-center">
            <div class="center-icon">
              <img
                src="/icons/Iconfactory-Ghost-In-The-Shell-Fuchikoma-Blue.32.png"
                alt="Tachikoma"
                class="center-tachi-icon"
              />
            </div>
            <div class="center-label">FORGE</div>
          </div>

          <!-- Connection Lines -->
          <svg class="connection-lines" viewBox="0 0 400 400">
            <defs>
              <linearGradient
                id="lineGradient"
                x1="0%"
                y1="0%"
                x2="100%"
                y2="0%"
              >
                <stop offset="0%" style="stop-color: rgba(78, 205, 196, 0.1)" />
                <stop
                  offset="50%"
                  style="stop-color: rgba(78, 205, 196, 0.4)"
                />
                <stop
                  offset="100%"
                  style="stop-color: rgba(78, 205, 196, 0.1)"
                />
              </linearGradient>
            </defs>
            {#each aiModels as _, i}
              {@const angle1 = (360 / aiModels.length) * i}
              {@const angle2 =
                (360 / aiModels.length) * ((i + 1) % aiModels.length)}
              {@const r = 140}
              {@const x1 = 200 + r * Math.cos(((angle1 - 90) * Math.PI) / 180)}
              {@const y1 = 200 + r * Math.sin(((angle1 - 90) * Math.PI) / 180)}
              {@const x2 = 200 + r * Math.cos(((angle2 - 90) * Math.PI) / 180)}
              {@const y2 = 200 + r * Math.sin(((angle2 - 90) * Math.PI) / 180)}
              <line
                {x1}
                {y1}
                {x2}
                {y2}
                stroke="url(#lineGradient)"
                stroke-width="1"
                class="connection-line"
                style="animation-delay: {i * 0.5}s"
              />
              <!-- Connection to center -->
              <line
                {x1}
                {y1}
                x2="200"
                y2="200"
                stroke="rgba(78, 205, 196, 0.15)"
                stroke-width="1"
                stroke-dasharray="4 4"
              />
            {/each}
          </svg>
        </div>

        <div class="council-description">
          <h2 class="description-title">MULTI-MODEL DELIBERATION</h2>
          <p class="description-text">
            The Think Tank (Spec Forge) orchestrates multiple AI backends
            through structured brainstorming rounds. Models draft specs,
            critique proposals, and synthesize improvements until convergence is
            reached.
          </p>
          {#if sacMode}
            <p class="lore-line">
              S.A.C. protocol engaged — independent agents, shared objective,
              emergent consensus.
            </p>
          {/if}
          <div class="forge-rounds">
            <div class="round-step">
              <span class="round-num">1</span> Initial Draft
            </div>
            <div class="round-arrow">→</div>
            <div class="round-step">
              <span class="round-num">2</span> Critique Round
            </div>
            <div class="round-arrow">→</div>
            <div class="round-step">
              <span class="round-num">3</span> Synthesis
            </div>
            <div class="round-arrow">→</div>
            <div class="round-step final">
              <span class="round-num">✓</span> Convergence
            </div>
          </div>
        </div>
      </GlassPanel>

      <!-- Features Grid -->
      <div class="features-grid">
        <GlassPanel className="feature-card" accent="red" subtle>
          <div class="feature-icon" style="--accent: #cc785c">
            <Icon name="file-plus" size={24} />
          </div>
          <h3 class="feature-title">SPEC DRAFTING</h3>
          <p class="feature-desc">
            Generate comprehensive specifications from high-level goals with
            multi-model consensus.
          </p>
        </GlassPanel>

        <GlassPanel className="feature-card" accent="green" subtle>
          <div class="feature-icon" style="--accent: #74aa9c">
            <Icon name="activity" size={24} />
          </div>
          <h3 class="feature-title">CRITIQUE ROUNDS</h3>
          <p class="feature-desc">
            Each model reviews and critiques proposals, identifying gaps and
            suggesting improvements.
          </p>
        </GlassPanel>

        <GlassPanel className="feature-card" accent="purple" subtle>
          <div class="feature-icon" style="--accent: #8b5cf6">
            <Icon name="refresh-cw" size={24} />
          </div>
          <h3 class="feature-title">ITERATIVE REFINEMENT</h3>
          <p class="feature-desc">
            Recursive improvement loops until convergence is reached and the
            spec meets quality thresholds.
          </p>
        </GlassPanel>

        <GlassPanel className="feature-card" accent="cyan" subtle>
          <div class="feature-icon" style="--accent: #4ecdc4">
            <Icon name="check-circle" size={24} />
          </div>
          <h3 class="feature-title">CONFLICT RESOLUTION</h3>
          <p class="feature-desc">
            The Oracle model mediates disagreements and makes final decisions
            with transparent rationale.
          </p>
        </GlassPanel>
      </div>

      <!-- Start Session CTA -->
      <div class="cta-section">
        <button class="start-session-btn" on:click={handleStartNewSession}>
          <div class="btn-bg"></div>
          <div class="btn-content">
            <Icon name="zap" size={24} />
            <span class="btn-text">INITIATE THINK TANK SESSION</span>
          </div>
          <div class="btn-glow"></div>
        </button>

        <p class="cta-hint">
          Start with a goal description, select AI participants, and let the
          Think Tank craft your specification.
        </p>
      </div>

      <!-- Past Sessions -->
      {#if $sessions.length > 0}
        <GlassPanel
          className="past-sessions"
          subtle
          accent="cyan"
          padded={false}
        >
          <div class="sessions-header">
            <Icon name="clock" size={18} />
            <span>PREVIOUS SESSIONS</span>
            <span class="session-count">{$sessions.length}</span>
          </div>
          <div class="sessions-body">
            <SessionSidebar sessionId={null} />
          </div>
        </GlassPanel>
      {/if}
    </div>
  {/if}
</div>

<style>
  .forge-page {
    max-width: 1200px;
    margin: 0 auto;
    display: flex;
    flex-direction: column;
    min-height: calc(100vh - 200px);
    position: relative;
  }

  .polyglass-backdrop {
    position: absolute;
    inset: -60px -40px -40px -40px;
    pointer-events: none;
    opacity: 0.7;
    filter: blur(0.2px);
    z-index: 0;
  }

  .polyglass-backdrop::before,
  .polyglass-backdrop::after {
    content: "";
    position: absolute;
    inset: 0;
    background:
      radial-gradient(
        circle at 15% 20%,
        rgba(78, 205, 196, 0.25),
        transparent 45%
      ),
      radial-gradient(
        circle at 80% 30%,
        rgba(139, 92, 246, 0.12),
        transparent 40%
      ),
      radial-gradient(
        circle at 65% 75%,
        rgba(255, 107, 107, 0.1),
        transparent 45%
      );
    opacity: 0.7;
  }

  .polyglass-backdrop::after {
    opacity: 0;
    background:
      linear-gradient(
        115deg,
        transparent 20%,
        rgba(78, 205, 196, 0.12) 45%,
        transparent 70%
      ),
      linear-gradient(160deg, rgba(255, 255, 255, 0.03), transparent);
    clip-path: polygon(10% 0, 70% 8%, 100% 55%, 62% 100%, 0 70%);
    transition: opacity 0.25s ease;
  }

  .polyglass-backdrop.enabled::after {
    opacity: 0.9;
  }

  /* Manga Panel Banner - Ghost in the Shell philosophical quote */
  .manga-banner {
    position: absolute;
    top: -10px;
    right: 0;
    width: 600px;
    height: 170px;
    background-image: url("/icons/C3-3.jpg");
    background-size: contain;
    background-position: right center;
    background-repeat: no-repeat;
    opacity: 0.35;
    mix-blend-mode: screen;
    mask-image: linear-gradient(
      to left,
      rgba(0, 0, 0, 1) 0%,
      rgba(0, 0, 0, 0.8) 50%,
      rgba(0, 0, 0, 0.3) 80%,
      transparent 100%
    );
    -webkit-mask-image: linear-gradient(
      to left,
      rgba(0, 0, 0, 1) 0%,
      rgba(0, 0, 0, 0.8) 50%,
      rgba(0, 0, 0, 0.3) 80%,
      transparent 100%
    );
    pointer-events: none;
    z-index: 0;
  }

  /* Responsive adjustments */
  @media (max-width: 900px) {
    .manga-banner {
      width: 400px;
      height: 110px;
    }
  }

  @media (max-width: 600px) {
    .manga-banner {
      width: 300px;
      height: 90px;
    }
  }

  /* Keep content above backdrop */
  :global(.forge-page > *) {
    position: relative;
    z-index: 1;
  }

  .loading-row {
    display: inline-flex;
    align-items: center;
    gap: 0.6rem;
    padding: 0.65rem 0.9rem;
    border-radius: 999px;
    background: rgba(13, 17, 23, 0.45);
    border: 1px solid rgba(78, 205, 196, 0.14);
    color: rgba(230, 237, 243, 0.75);
    font-size: 0.9rem;
    margin: 0 auto;
  }

  .error-panel {
    margin-bottom: 1.25rem;
  }

  .error-content {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 1rem;
  }

  .error-left {
    display: flex;
    align-items: center;
    gap: 0.75rem;
  }

  .error-text {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
  }

  .error-title {
    font-family: var(--font-display, "Orbitron", sans-serif);
    letter-spacing: 1px;
    text-transform: uppercase;
    font-size: 0.8rem;
    color: rgba(230, 237, 243, 0.9);
  }

  .error-subtitle {
    color: rgba(230, 237, 243, 0.65);
    font-size: 0.9rem;
  }

  .error-actions {
    display: flex;
    align-items: center;
    gap: 0.6rem;
  }

  /* Think Tank Welcome */
  .think-tank-welcome {
    display: flex;
    flex-direction: column;
    gap: 3rem;
  }

  /* Council Visualization */
  .council-section {
    display: flex;
    align-items: center;
    gap: 3rem;
    position: relative;
    overflow: hidden;
  }

  .council-section::before {
    content: "";
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    height: 3px;
    background: linear-gradient(
      90deg,
      transparent,
      var(--tachi-cyan, #4ecdc4) 30%,
      var(--tachi-cyan, #4ecdc4) 70%,
      transparent
    );
  }

  .council-ring {
    position: relative;
    width: 400px;
    height: 400px;
    flex-shrink: 0;
  }

  .ring-glow {
    position: absolute;
    top: 50%;
    left: 50%;
    transform: translate(-50%, -50%);
    width: 320px;
    height: 320px;
    background: radial-gradient(
      circle,
      rgba(78, 205, 196, 0.1) 0%,
      transparent 70%
    );
    border-radius: 50%;
    animation: ringPulse 4s ease-in-out infinite;
  }

  @keyframes ringPulse {
    0%,
    100% {
      opacity: 0.5;
      transform: translate(-50%, -50%) scale(1);
    }
    50% {
      opacity: 1;
      transform: translate(-50%, -50%) scale(1.05);
    }
  }

  .ring-track {
    position: absolute;
    top: 50%;
    left: 50%;
    transform: translate(-50%, -50%);
    width: 280px;
    height: 280px;
    border: 1px solid rgba(78, 205, 196, 0.2);
    border-radius: 50%;
  }

  .council-node {
    position: absolute;
    top: 50%;
    left: 50%;
    transform: rotate(var(--angle)) translateY(-140px)
      rotate(calc(-1 * var(--angle)));
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0.5rem;

    /* Orbit to reduce static overlaps (e.g. Gemini over the header) */
    animation: councilOrbit 22s linear infinite;
    animation-delay: var(--delay, 0s);
    will-change: transform;
  }

  @keyframes councilOrbit {
    from {
      transform: rotate(var(--angle)) translateY(-140px)
        rotate(calc(-1 * var(--angle)));
    }
    to {
      transform: rotate(calc(var(--angle) + 360deg)) translateY(-140px)
        rotate(calc(-1 * (var(--angle) + 360deg)));
    }
  }

  .node-avatar {
    width: 56px;
    height: 56px;
    display: flex;
    align-items: center;
    justify-content: center;
    background: linear-gradient(
      135deg,
      rgba(0, 0, 0, 0.8),
      var(--bg-tertiary, #1c2128)
    );
    border: 2px solid var(--node-color);
    border-radius: 50%;
    color: var(--node-color);
    box-shadow: 0 0 20px color-mix(in srgb, var(--node-color) 40%, transparent);
    transition: all 0.3s ease;
  }

  .tachi-model-icon {
    width: 36px;
    height: 36px;
    object-fit: contain;
    filter: drop-shadow(0 0 4px var(--node-color));
  }

  .council-node:hover .tachi-model-icon {
    filter: drop-shadow(0 0 8px var(--node-color))
      drop-shadow(0 0 4px var(--node-color));
  }

  .council-node:hover .node-avatar {
    transform: scale(1.1);
    box-shadow: 0 0 30px color-mix(in srgb, var(--node-color) 60%, transparent);
  }

  .node-info {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0;
  }

  .node-name {
    font-family: var(--font-display, "Orbitron", sans-serif);
    font-size: 0.7rem;
    font-weight: 700;
    color: var(--node-color);
    letter-spacing: 1px;
  }

  .node-role {
    font-size: 0.6rem;
    color: var(--text-muted, rgba(230, 237, 243, 0.5));
    letter-spacing: 0.5px;
  }

  .node-pulse {
    position: absolute;
    top: 0;
    left: 50%;
    transform: translateX(-50%);
    width: 56px;
    height: 56px;
    border: 1px solid var(--node-color);
    border-radius: 50%;
    animation: nodePulse 2s ease-out infinite;
    animation-delay: calc(var(--angle) / 360 * 2s);
  }

  @keyframes nodePulse {
    0% {
      opacity: 0.6;
      transform: translateX(-50%) scale(1);
    }
    100% {
      opacity: 0;
      transform: translateX(-50%) scale(1.5);
    }
  }

  .council-center {
    position: absolute;
    top: 50%;
    left: 50%;
    transform: translate(-50%, -50%);
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0.5rem;
  }

  .center-icon {
    width: 72px;
    height: 72px;
    display: flex;
    align-items: center;
    justify-content: center;
    background: linear-gradient(
      135deg,
      var(--bg-tertiary, #1c2128),
      var(--bg-primary, #0d1117)
    );
    border: 2px solid var(--tachi-cyan, #4ecdc4);
    border-radius: 50%;
    color: var(--tachi-cyan, #4ecdc4);
    box-shadow:
      0 0 30px rgba(78, 205, 196, 0.4),
      inset 0 0 20px rgba(78, 205, 196, 0.1);
    animation: centerGlow 3s ease-in-out infinite;
  }

  .center-tachi-icon {
    width: 48px;
    height: 48px;
    object-fit: contain;
    filter: drop-shadow(0 0 8px var(--tachi-cyan, #4ecdc4));
    animation: tachiGlow 3s ease-in-out infinite;
  }

  @keyframes tachiGlow {
    0%,
    100% {
      filter: drop-shadow(0 0 8px var(--tachi-cyan, #4ecdc4));
    }
    50% {
      filter: drop-shadow(0 0 14px var(--tachi-cyan, #4ecdc4))
        drop-shadow(0 0 4px var(--tachi-cyan, #4ecdc4));
    }
  }

  @keyframes centerGlow {
    0%,
    100% {
      box-shadow:
        0 0 30px rgba(78, 205, 196, 0.4),
        inset 0 0 20px rgba(78, 205, 196, 0.1);
    }
    50% {
      box-shadow:
        0 0 50px rgba(78, 205, 196, 0.6),
        inset 0 0 30px rgba(78, 205, 196, 0.2);
    }
  }

  .center-label {
    font-family: var(--font-display, "Orbitron", sans-serif);
    font-size: 0.7rem;
    font-weight: 700;
    color: var(--tachi-cyan, #4ecdc4);
    letter-spacing: 2px;
    text-shadow: 0 0 10px var(--tachi-cyan, #4ecdc4);
  }

  .connection-lines {
    position: absolute;
    inset: 0;
    pointer-events: none;
  }

  .connection-line {
    animation: lineFlow 2s ease-in-out infinite;
  }

  @keyframes lineFlow {
    0%,
    100% {
      opacity: 0.3;
    }
    50% {
      opacity: 0.8;
    }
  }

  .council-description {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

  .description-title {
    font-family: var(--font-display, "Orbitron", sans-serif);
    font-size: 1.5rem;
    font-weight: 700;
    color: var(--text-primary, #e6edf3);
    letter-spacing: 2px;
    margin: 0;
    text-shadow: 0 0 20px var(--tachi-cyan-glow, rgba(78, 205, 196, 0.3));
  }

  .description-text {
    font-size: 1.05rem;
    color: var(--text-secondary, rgba(230, 237, 243, 0.7));
    line-height: 1.7;
    margin: 0;
  }

  .lore-line {
    margin: 0.25rem 0 0;
    padding-left: 0.75rem;
    border-left: 2px solid rgba(78, 205, 196, 0.4);
    color: rgba(230, 237, 243, 0.6);
    font-style: italic;
  }

  .forge-rounds {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    margin-top: 1.5rem;
    flex-wrap: wrap;
  }

  .round-step {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.5rem 0.75rem;
    background: var(--bg-tertiary, #1c2128);
    border: 1px solid var(--border-color, rgba(78, 205, 196, 0.2));
    border-radius: 6px;
    font-family: var(--font-display, "Orbitron", sans-serif);
    font-size: 0.7rem;
    font-weight: 500;
    color: var(--text-secondary, rgba(230, 237, 243, 0.7));
    letter-spacing: 0.5px;
  }

  .round-step.final {
    border-color: var(--success-color, #3fb950);
    color: var(--success-color, #3fb950);
  }

  .round-num {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 20px;
    height: 20px;
    background: rgba(78, 205, 196, 0.2);
    border-radius: 50%;
    font-size: 0.65rem;
    color: var(--tachi-cyan, #4ecdc4);
  }

  .round-step.final .round-num {
    background: rgba(63, 185, 80, 0.2);
    color: var(--success-color, #3fb950);
  }

  .round-arrow {
    color: var(--text-muted, rgba(230, 237, 243, 0.3));
    font-size: 0.8rem;
  }

  /* Features Grid */
  .features-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
    gap: 1rem;
  }

  .feature-card {
    transition: all 0.3s ease;
    position: relative;
    overflow: hidden;
  }

  .feature-card::before {
    content: "";
    position: absolute;
    top: 0;
    left: 0;
    width: 100%;
    height: 2px;
    background: linear-gradient(90deg, var(--accent, #4ecdc4), transparent);
    opacity: 0;
    transition: opacity 0.3s ease;
  }

  .feature-card:hover {
    border-color: var(--tachi-cyan, #4ecdc4);
    transform: translateY(-2px);
  }

  .feature-card:hover::before {
    opacity: 1;
  }

  .feature-icon {
    width: 48px;
    height: 48px;
    display: flex;
    align-items: center;
    justify-content: center;
    background: linear-gradient(
      135deg,
      color-mix(in srgb, var(--accent) 20%, transparent),
      transparent
    );
    border: 1px solid color-mix(in srgb, var(--accent) 40%, transparent);
    border-radius: 10px;
    color: var(--accent);
    margin-bottom: 1rem;
  }

  .feature-title {
    font-family: var(--font-display, "Orbitron", sans-serif);
    font-size: 0.8rem;
    font-weight: 600;
    color: var(--text-primary, #e6edf3);
    letter-spacing: 1px;
    margin: 0 0 0.5rem;
  }

  .feature-desc {
    font-size: 0.9rem;
    color: var(--text-muted, rgba(230, 237, 243, 0.6));
    line-height: 1.5;
    margin: 0;
  }

  /* CTA Section */
  .cta-section {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 1rem;
    padding: 2rem;
  }

  .start-session-btn {
    position: relative;
    padding: 1.25rem 3rem;
    background: transparent;
    border: 2px solid var(--tachi-cyan, #4ecdc4);
    border-radius: 12px;
    cursor: pointer;
    overflow: hidden;
    transition: all 0.3s ease;
  }

  .start-session-btn .btn-bg {
    position: absolute;
    inset: 0;
    background: linear-gradient(
      135deg,
      var(--tachi-cyan-dark, #2d7a7a),
      var(--tachi-cyan, #4ecdc4)
    );
    opacity: 0;
    transition: opacity 0.3s ease;
  }

  .start-session-btn:hover .btn-bg {
    opacity: 1;
  }

  .start-session-btn .btn-content {
    position: relative;
    display: flex;
    align-items: center;
    gap: 0.75rem;
    color: var(--tachi-cyan, #4ecdc4);
    transition: color 0.3s ease;
  }

  .start-session-btn:hover .btn-content {
    color: var(--bg-primary, #0d1117);
  }

  .start-session-btn .btn-text {
    font-family: var(--font-display, "Orbitron", sans-serif);
    font-size: 0.9rem;
    font-weight: 700;
    letter-spacing: 1.5px;
  }

  .start-session-btn .btn-glow {
    position: absolute;
    inset: -2px;
    border-radius: 14px;
    background: transparent;
    box-shadow: 0 0 20px rgba(78, 205, 196, 0.3);
    opacity: 0;
    transition: opacity 0.3s ease;
  }

  .start-session-btn:hover .btn-glow {
    opacity: 1;
    animation: btnGlowPulse 1.5s ease-in-out infinite;
  }

  @keyframes btnGlowPulse {
    0%,
    100% {
      box-shadow: 0 0 20px rgba(78, 205, 196, 0.3);
    }
    50% {
      box-shadow: 0 0 40px rgba(78, 205, 196, 0.5);
    }
  }

  .cta-hint {
    font-size: 0.9rem;
    color: var(--text-muted, rgba(230, 237, 243, 0.5));
    text-align: center;
    max-width: 500px;
    margin: 0;
  }

  /* Past Sessions */
  .past-sessions {
    overflow: hidden;
  }

  .sessions-body {
    padding: 1rem 1rem 1.25rem;
  }

  .sessions-header {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding: 1rem 1.25rem;
    background: linear-gradient(90deg, rgba(78, 205, 196, 0.08), transparent);
    border-bottom: 1px solid var(--border-color, rgba(78, 205, 196, 0.1));
    color: var(--tachi-cyan, #4ecdc4);
    font-family: var(--font-display, "Orbitron", sans-serif);
    font-size: 0.75rem;
    font-weight: 600;
    letter-spacing: 1.5px;
  }

  .session-count {
    margin-left: auto;
    padding: 0.25rem 0.5rem;
    background: rgba(78, 205, 196, 0.15);
    border-radius: 4px;
    font-size: 0.7rem;
  }

  /* Responsive */
  @media (max-width: 900px) {
    .council-section {
      flex-direction: column;
    }

    .council-ring {
      width: 320px;
      height: 320px;
    }

    .council-node {
      transform: rotate(var(--angle)) translateY(-110px)
        rotate(calc(-1 * var(--angle)));
    }

    .ring-track {
      width: 220px;
      height: 220px;
    }
  }

  /* Respect reduced motion: stop orbit + avoid loading visual noise */
  @media (prefers-reduced-motion: reduce) {
    .council-node {
      animation: none;
    }
  }
</style>
