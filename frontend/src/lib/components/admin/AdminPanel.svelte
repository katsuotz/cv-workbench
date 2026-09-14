<script lang="ts">
  import { onMount } from 'svelte';

  import { BackendApiError, createBackendApi, type AdminUser, type AuthUser } from '$lib/api';
  import { Button, ButtonLink } from '../base';

  type PageState = 'loading' | 'ready' | 'signed-out' | 'forbidden' | 'error';

  const api = createBackendApi();

  let pageState: PageState = 'loading';
  let authUser: AuthUser | null = null;
  let users: AdminUser[] = [];
  let search = '';
  let error = '';
  let listLoading = false;

  $: normalizedSearch = search.trim().toLowerCase();
  $: filteredUsers = normalizedSearch
    ? users.filter((user) =>
        [user.email, user.name ?? '', user.role, user.id].some((value) =>
          value.toLowerCase().includes(normalizedSearch)
        )
      )
    : users;

  onMount(() => {
    void loadPage();
  });

  async function loadPage() {
    pageState = 'loading';
    error = '';

    try {
      authUser = await api.auth.getCurrentUser();
      if (!authUser) {
        pageState = 'signed-out';
        return;
      }
      if (authUser.role !== 'root') {
        pageState = 'forbidden';
        return;
      }
      await loadUsers();
      pageState = 'ready';
    } catch (cause) {
      error = getErrorMessage(cause, 'Could not load the admin panel.');
      pageState = 'error';
    }
  }

  async function loadUsers() {
    listLoading = true;
    error = '';
    try {
      users = await api.admin.listUsers();
    } catch (cause) {
      error = getErrorMessage(cause, 'Could not load users.');
      throw cause;
    } finally {
      listLoading = false;
    }
  }

  async function refreshUsers() {
    if (listLoading) return;
    try {
      await loadUsers();
    } catch {
      pageState = 'error';
    }
  }

  function getErrorMessage(cause: unknown, fallback: string) {
    if (cause instanceof BackendApiError && cause.status === 403) {
      return 'This account does not have access to the admin panel.';
    }
    if (cause instanceof BackendApiError && cause.status >= 500) {
      return 'The server could not return the user list. Try again.';
    }
    return cause instanceof Error ? cause.message : fallback;
  }

  function formatDate(value: string) {
    return new Intl.DateTimeFormat(undefined, {
      year: 'numeric',
      month: 'short',
      day: 'numeric'
    }).format(new Date(value));
  }

  function initials(user: AdminUser) {
    const source = user.name?.trim() || user.email;
    return source
      .split(/[\s@._-]+/)
      .filter(Boolean)
      .slice(0, 2)
      .map((part) => part[0])
      .join('')
      .toUpperCase();
  }

  function shortId(id: string) {
    return `${id.slice(0, 8)}…${id.slice(-4)}`;
  }
</script>

<div class="admin-shell">
  <header class="admin-header">
    <div class="admin-header-inner">
      <a class="brand-lockup" href="/app" aria-label="CV Workbench builder">
        <span class="brand-mark" aria-hidden="true">CV</span>
        <span class="brand-name">CV Workbench</span>
      </a>
      <div class="header-context" aria-label="Current admin section">
        <span class="context-separator" aria-hidden="true">/</span>
        <span>Admin</span>
        <span class="context-separator" aria-hidden="true">/</span>
        <strong>Users</strong>
      </div>
      <div class="header-actions">
        {#if authUser}<span class="signed-in-user">{authUser.email}</span>{/if}
        <ButtonLink href="/app" variant="secondary" className="back-link">
          Back to builder
        </ButtonLink>
      </div>
    </div>
  </header>

  <main class="admin-main">
    {#if pageState === 'loading'}
      <section class="access-state" aria-live="polite" aria-busy="true">
        <div class="loading-line loading-title"></div>
        <div class="loading-line loading-copy"></div>
        <div class="loading-table" aria-hidden="true">
          {#each Array(4) as _}
            <div class="loading-row">
              <span></span>
              <span></span>
              <span></span>
            </div>
          {/each}
        </div>
      </section>
    {:else if pageState === 'signed-out'}
      <section class="access-state" aria-labelledby="signed-out-title">
        <p class="state-code">AUTH / REQUIRED</p>
        <h1 id="signed-out-title">Sign in to view users</h1>
        <p>Your account needs root access before the user directory can be opened.</p>
        <ButtonLink href="/app" variant="primary">Open builder</ButtonLink>
      </section>
    {:else if pageState === 'forbidden'}
      <section class="access-state" aria-labelledby="forbidden-title">
        <p class="state-code">AUTH / FORBIDDEN</p>
        <h1 id="forbidden-title">Admin access is restricted</h1>
        <p>The signed-in account is not a root account.</p>
        <ButtonLink href="/app" variant="secondary">Back to builder</ButtonLink>
      </section>
    {:else if pageState === 'error'}
      <section class="access-state" aria-labelledby="error-title">
        <p class="state-code">REQUEST / FAILED</p>
        <h1 id="error-title">The admin panel is unavailable</h1>
        <p>{error}</p>
        <Button variant="primary" onClick={loadPage}>Try again</Button>
      </section>
    {:else}
      <section class="page-heading" aria-labelledby="users-title">
        <div>
          <h1 id="users-title">Users</h1>
          <p class="heading-copy">Review the accounts that can access CV Workbench.</p>
        </div>
        <div class="list-count" aria-live="polite">
          <strong>{filteredUsers.length}</strong>
          <span>{normalizedSearch ? 'matching users' : 'total users'}</span>
        </div>
      </section>

      <section class="users-surface" aria-labelledby="users-table-title">
        <div class="surface-toolbar">
          <div>
            <h2 id="users-table-title">Account directory</h2>
            <p>Search by name, email, role, or account ID.</p>
          </div>
          <div class="toolbar-actions">
            <label class="search-field" for="user-search">
              <span>Filter users</span>
              <input
                id="user-search"
                type="search"
                bind:value={search}
                placeholder="Search users"
                autocomplete="off" />
            </label>
            <Button variant="secondary" loading={listLoading} onClick={refreshUsers}>
              Refresh
            </Button>
          </div>
        </div>

        {#if error}
          <p class="inline-error" role="alert">{error}</p>
        {/if}

        {#if users.length === 0 && !listLoading}
          <div class="table-state">
            <h3>No user accounts found</h3>
            <p>The directory is empty.</p>
          </div>
        {:else if filteredUsers.length === 0}
          <div class="table-state">
            <h3>No matching users</h3>
            <p>Try a different name, email, role, or account ID.</p>
          </div>
        {:else}
          <div class="table-scroll">
            <table>
              <caption class="sr-only">CV Workbench user accounts</caption>
              <thead>
                <tr>
                  <th scope="col">Account</th>
                  <th scope="col">Role</th>
                  <th scope="col">Joined</th>
                  <th scope="col">Account ID</th>
                </tr>
              </thead>
              <tbody>
                {#each filteredUsers as user (user.id)}
                  <tr>
                    <td data-label="Account">
                      <div class="user-cell">
                        <span class="avatar" aria-hidden="true">{initials(user)}</span>
                        <span class="user-details">
                          <strong>{user.name || 'No name provided'}</strong>
                          <span>{user.email}</span>
                        </span>
                      </div>
                    </td>
                    <td data-label="Role">
                      <span class:role-root={user.role === 'root'} class="role-label">
                        <span class="role-dot" aria-hidden="true"></span>
                        {user.role === 'root' ? 'Root' : 'User'}
                      </span>
                    </td>
                    <td data-label="Joined">
                      <time datetime={user.created_at}>{formatDate(user.created_at)}</time>
                    </td>
                    <td data-label="Account ID"><code title={user.id}>{shortId(user.id)}</code></td>
                  </tr>
                {/each}
              </tbody>
            </table>
          </div>
        {/if}
      </section>
    {/if}
  </main>
</div>

<style>
  :global(body) {
    overflow: auto;
  }

  .admin-shell {
    min-height: 100vh;
    color: var(--ink);
  }

  .admin-header {
    border-bottom: 1px solid var(--rule);
    background: var(--surface);
  }

  .admin-header-inner {
    display: grid;
    min-height: 68px;
    grid-template-columns: auto 1fr auto;
    align-items: center;
    gap: 28px;
    width: min(calc(100% - 56px), 1320px);
    margin: 0 auto;
  }

  .brand-lockup {
    display: inline-flex;
    align-items: center;
    gap: 11px;
    color: var(--ink);
    text-decoration: none;
  }

  .brand-mark {
    display: grid;
    width: 30px;
    height: 30px;
    place-items: center;
    border: 1px solid var(--blue);
    color: var(--blue);
    font-size: 14px;
    font-weight: 700;
    letter-spacing: -0.05em;
  }

  .brand-name {
    font-size: 16px;
    font-weight: 700;
    letter-spacing: -0.02em;
  }

  .header-context,
  .header-actions,
  .toolbar-actions {
    display: flex;
    align-items: center;
  }

  .header-context {
    gap: 10px;
    color: var(--muted-ink);
    font-family: var(--mono);
    font-size: 11px;
    letter-spacing: 0.05em;
    text-transform: uppercase;
  }

  .header-context strong {
    color: var(--ink);
    font-weight: 600;
  }

  .context-separator {
    color: var(--rule-strong);
  }

  .header-actions {
    justify-content: flex-end;
    gap: 16px;
  }

  .signed-in-user {
    max-width: 220px;
    overflow: hidden;
    color: var(--muted-ink);
    font-family: var(--mono);
    font-size: 11px;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .admin-main {
    width: min(calc(100% - 56px), 1320px);
    margin: 0 auto;
    padding: 56px 0 80px;
  }

  .page-heading {
    display: flex;
    align-items: flex-end;
    justify-content: space-between;
    gap: 32px;
    margin-bottom: 34px;
  }

  .state-code {
    margin: 0 0 9px;
    color: var(--blue-dark);
    font-family: var(--mono);
    font-size: 11px;
    font-weight: 600;
    letter-spacing: 0.07em;
  }

  h1,
  h2,
  h3,
  p {
    margin-top: 0;
  }

  h1 {
    margin-bottom: 9px;
    font-size: clamp(32px, 4vw, 48px);
    letter-spacing: -0.045em;
    line-height: 1;
  }

  .heading-copy {
    max-width: 58ch;
    margin-bottom: 0;
    color: var(--muted-ink);
    font-size: 15px;
    line-height: 1.5;
  }

  .list-count {
    display: grid;
    gap: 3px;
    min-width: 108px;
    padding-bottom: 2px;
    text-align: right;
  }

  .list-count strong {
    font-family: var(--mono);
    font-size: 25px;
    font-weight: 600;
    letter-spacing: -0.05em;
  }

  .list-count span {
    color: var(--quiet-ink);
    font-family: var(--mono);
    font-size: 10px;
    letter-spacing: 0.05em;
    text-transform: uppercase;
  }

  .users-surface {
    overflow: hidden;
    border: 1px solid var(--rule);
    border-radius: 12px;
    background: var(--surface);
  }

  .surface-toolbar {
    display: flex;
    align-items: flex-end;
    justify-content: space-between;
    gap: 28px;
    padding: 22px 24px;
    border-bottom: 1px solid var(--rule);
    background: var(--surface-subtle);
  }

  .surface-toolbar h2 {
    margin-bottom: 5px;
    font-size: 17px;
    letter-spacing: -0.025em;
  }

  .surface-toolbar p {
    margin-bottom: 0;
    color: var(--muted-ink);
    font-size: 13px;
  }

  .toolbar-actions {
    gap: 12px;
  }

  .search-field {
    display: grid;
    gap: 6px;
    min-width: min(280px, 32vw);
  }

  .search-field span {
    color: var(--muted-ink);
    font-family: var(--mono);
    font-size: 10px;
    letter-spacing: 0.06em;
    text-transform: uppercase;
  }

  .search-field input {
    width: 100%;
    min-height: 36px;
    border: 1px solid var(--rule-strong);
    border-radius: 7px;
    padding: 0 11px;
    background: var(--surface);
    color: var(--ink);
    font: inherit;
    font-size: 13px;
  }

  .search-field input::placeholder {
    color: var(--quiet-ink);
  }

  .search-field input:focus {
    border-color: var(--blue);
    outline: 2px solid var(--blue-soft);
    outline-offset: 1px;
  }

  .inline-error {
    margin: 0;
    border-bottom: 1px solid var(--rule);
    padding: 13px 24px;
    background: var(--danger-soft);
    color: var(--danger);
    font-size: 13px;
  }

  .table-scroll {
    overflow-x: auto;
  }

  table {
    width: 100%;
    border-collapse: collapse;
    text-align: left;
  }

  th,
  td {
    padding: 15px 24px;
    border-bottom: 1px solid var(--rule);
    vertical-align: middle;
  }

  th {
    color: var(--quiet-ink);
    font-family: var(--mono);
    font-size: 10px;
    font-weight: 600;
    letter-spacing: 0.06em;
    text-transform: uppercase;
  }

  tbody tr:last-child td {
    border-bottom: 0;
  }

  tbody tr:hover {
    background: var(--surface-raised);
  }

  .user-cell {
    display: flex;
    align-items: center;
    gap: 12px;
    min-width: 260px;
  }

  .avatar {
    display: grid;
    flex: 0 0 34px;
    width: 34px;
    height: 34px;
    place-items: center;
    border: 1px solid var(--rule-strong);
    border-radius: 50%;
    background: var(--blue-soft);
    color: var(--blue-dark);
    font-family: var(--mono);
    font-size: 10px;
    font-weight: 600;
  }

  .user-details {
    display: grid;
    gap: 3px;
  }

  .user-details strong {
    font-size: 13px;
    font-weight: 650;
  }

  .user-details span,
  time,
  code {
    color: var(--muted-ink);
    font-family: var(--mono);
    font-size: 11px;
  }

  .role-label {
    display: inline-flex;
    align-items: center;
    gap: 7px;
    color: var(--muted-ink);
    font-family: var(--mono);
    font-size: 11px;
  }

  .role-dot {
    width: 7px;
    height: 7px;
    border-radius: 50%;
    background: var(--rule-strong);
  }

  .role-root {
    color: var(--blue-dark);
  }

  .role-root .role-dot {
    background: var(--blue);
  }

  code {
    white-space: nowrap;
  }

  .table-state,
  .access-state {
    padding: 52px 24px;
  }

  .table-state {
    text-align: center;
  }

  .table-state h3 {
    margin-bottom: 6px;
    font-size: 15px;
  }

  .table-state p,
  .access-state > p:not(.state-code) {
    margin-bottom: 20px;
    color: var(--muted-ink);
    font-size: 14px;
    line-height: 1.5;
  }

  .access-state {
    max-width: 560px;
    margin: 60px auto;
    border: 1px solid var(--rule);
    border-radius: 12px;
    background: var(--surface);
  }

  .access-state h1 {
    font-size: clamp(28px, 4vw, 42px);
    line-height: 1.05;
  }

  .loading-line,
  .loading-row span {
    background: var(--surface-subtle);
    animation: admin-pulse 1.2s ease-in-out infinite alternate;
  }

  .loading-title {
    width: min(280px, 70%);
    height: 44px;
    margin-bottom: 12px;
  }

  .loading-copy {
    width: min(440px, 90%);
    height: 18px;
    margin-bottom: 36px;
  }

  .loading-table {
    overflow: hidden;
    border: 1px solid var(--rule);
    border-radius: 12px;
  }

  .loading-row {
    display: grid;
    grid-template-columns: 2fr 1fr 1fr;
    gap: 22px;
    padding: 22px 24px;
    border-bottom: 1px solid var(--rule);
  }

  .loading-row:last-child {
    border-bottom: 0;
  }

  .loading-row span {
    height: 13px;
  }

  @keyframes admin-pulse {
    from {
      opacity: 0.48;
    }
    to {
      opacity: 1;
    }
  }

  .sr-only {
    position: absolute;
    width: 1px;
    height: 1px;
    overflow: hidden;
    padding: 0;
    border: 0;
    margin: -1px;
    clip: rect(0, 0, 0, 0);
    white-space: nowrap;
  }

  @media (max-width: 820px) {
    .admin-header-inner {
      grid-template-columns: auto 1fr;
      width: min(calc(100% - 32px), 680px);
      gap: 14px 20px;
      padding: 12px 0;
    }

    .header-context {
      grid-column: 1 / -1;
      grid-row: 2;
      order: 3;
    }

    .header-actions {
      grid-column: 2;
      grid-row: 1;
    }

    .signed-in-user {
      max-width: 140px;
    }

    .admin-main {
      width: min(calc(100% - 32px), 680px);
      padding-top: 38px;
    }

    .surface-toolbar {
      align-items: stretch;
      flex-direction: column;
      gap: 20px;
    }

    .toolbar-actions {
      align-items: flex-end;
    }

    .search-field {
      flex: 1;
      min-width: 0;
    }
  }

  @media (max-width: 560px) {
    .brand-name,
    .signed-in-user {
      display: none;
    }

    .header-actions {
      gap: 0;
    }

    .page-heading {
      align-items: flex-start;
      flex-direction: column;
      gap: 20px;
      margin-bottom: 26px;
    }

    .list-count {
      text-align: left;
    }

    .toolbar-actions {
      align-items: stretch;
      flex-direction: column;
    }

    .toolbar-actions :global(.button) {
      width: 100%;
    }

    .table-scroll {
      overflow: visible;
    }

    table,
    tbody,
    tr,
    td {
      display: block;
    }

    thead {
      position: absolute;
      width: 1px;
      height: 1px;
      overflow: hidden;
      clip: rect(0, 0, 0, 0);
    }

    tbody tr {
      padding: 17px 20px;
      border-bottom: 1px solid var(--rule);
    }

    tbody tr:last-child {
      border-bottom: 0;
    }

    td {
      display: grid;
      grid-template-columns: 82px 1fr;
      gap: 12px;
      padding: 6px 0;
      border: 0;
    }

    td::before {
      content: attr(data-label);
      color: var(--quiet-ink);
      font-family: var(--mono);
      font-size: 10px;
      letter-spacing: 0.05em;
      text-transform: uppercase;
    }

    td:first-child {
      padding-top: 0;
    }

    td:last-child {
      padding-bottom: 0;
    }

    .user-cell {
      min-width: 0;
    }

    .access-state {
      margin: 24px 0;
      padding: 32px 20px;
    }
  }
</style>
