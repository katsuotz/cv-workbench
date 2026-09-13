import { createServer } from 'node:http';

const port = 18733;
const documents = new Map();
const jobs = new Map();
const cvSessions = new Map();
const pendingImports = new Map();
const users = new Map();
let sequence = 0;
const templates = [
  {
    id: 'editorial-v1',
    name: 'Editorial dossier',
    description: 'A quiet, structured page for thoughtful work.'
  },
  {
    id: 'compact-v1',
    name: 'Compact signal',
    description: 'A denser layout for broad experience.'
  },
  {
    id: 'modern-v1',
    name: 'Modern hierarchy',
    description: 'A contemporary layout with a stronger accent.'
  }
];

function id(prefix) {
  sequence += 1;
  return `${prefix}-${sequence}`;
}

function makeUser(email, name = null) {
  return {
    id: id('user'),
    email,
    name,
    role: email === 'root@example.test' ? 'root' : 'user'
  };
}

function makePdf() {
  const stream = 'BT\n/F1 24 Tf\n72 720 Td\n(Backend compiled CV) Tj\nET\n';
  const objects = [
    '<< /Type /Catalog /Pages 2 0 R >>',
    '<< /Type /Pages /Kids [3 0 R] /Count 1 >>',
    '<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] /Resources << /Font << /F1 5 0 R >> >> /Contents 4 0 R >>',
    `<< /Length ${Buffer.byteLength(stream)} >>\nstream\n${stream}endstream`,
    '<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>'
  ];
  let body = '%PDF-1.4\n';
  const offsets = [0];
  for (const [index, object] of objects.entries()) {
    offsets.push(Buffer.byteLength(body));
    body += `${index + 1} 0 obj\n${object}\nendobj\n`;
  }
  const xref = Buffer.byteLength(body);
  body += `xref\n0 ${objects.length + 1}\n0000000000 65535 f \n`;
  body += offsets
    .slice(1)
    .map((offset) => `${String(offset).padStart(10, '0')} 00000 n \n`)
    .join('');
  body += `trailer\n<< /Size ${objects.length + 1} /Root 1 0 R >>\nstartxref\n${xref}\n%%EOF\n`;
  return Buffer.from(body);
}

const pdf = makePdf();

function json(response, status, value) {
  response.writeHead(status, {
    'Access-Control-Allow-Credentials': 'true',
    'Access-Control-Allow-Origin': 'http://127.0.0.1:5173',
    'Content-Type': 'application/json'
  });
  response.end(JSON.stringify(value));
}

async function body(request) {
  const chunks = [];
  for await (const chunk of request) chunks.push(chunk);
  return chunks.length ? JSON.parse(Buffer.concat(chunks).toString('utf8')) : {};
}

const server = createServer(async (request, response) => {
  const url = new URL(request.url ?? '/', `http://${request.headers.host}`);
  const path = url.pathname;

  if (request.method === 'OPTIONS') {
    response.writeHead(204, {
      'Access-Control-Allow-Credentials': 'true',
      'Access-Control-Allow-Headers': 'Authorization, Content-Type, Accept',
      'Access-Control-Allow-Methods': 'GET, POST, PUT, DELETE, OPTIONS',
      'Access-Control-Allow-Origin': 'http://127.0.0.1:5173'
    });
    response.end();
    return;
  }
  if (request.method === 'GET' && path === '/') {
    json(response, 200, { status: 'ok' });
    return;
  }
  if (request.method === 'POST' && path === '/api/v1/sessions/anonymous') {
    const currentSessionId = request.headers.cookie?.match(/lr_session=([^;]+)/)?.[1];
    const sessionId =
      currentSessionId && !users.has(currentSessionId) ? currentSessionId : id('session');
    response.setHeader('Set-Cookie', `lr_session=${sessionId}; Path=/; SameSite=Lax`);
    json(response, 201, {
      session_id: sessionId,
      token: 'e2e-token',
      expires_at: '2099-01-01T00:00:00Z'
    });
    return;
  }

  if (request.method === 'GET' && path === '/api/v1/cv/templates') {
    json(response, 200, { templates });
    return;
  }
  const templatePreviewPath = path.match(/^\/api\/v1\/cv\/templates\/([^/]+)\/preview$/);
  if (request.method === 'GET' && templatePreviewPath) {
    response.writeHead(200, {
      'Access-Control-Allow-Credentials': 'true',
      'Access-Control-Allow-Origin': 'http://127.0.0.1:5173',
      'Content-Length': pdf.byteLength,
      'Content-Type': 'application/pdf'
    });
    response.end(pdf);
    return;
  }

  if (request.method === 'GET' && path === '/api/v1/auth/me') {
    const user = users.get(request.headers.cookie?.match(/lr_session=([^;]+)/)?.[1]);
    json(
      response,
      user ? 200 : 401,
      user ? { user } : { code: 'unauthorized', message: 'Not signed in.' }
    );
    return;
  }
  if (request.method === 'GET' && path === '/api/v1/auth/google/start') {
    const previousToken = request.headers.cookie?.match(/lr_session=([^;]+)/)?.[1];
    const user = makeUser('google@example.com', 'Google User');
    const userToken = id('user-session');
    users.set(userToken, user);
    if (previousToken && cvSessions.has(previousToken)) {
      cvSessions.set(userToken, cvSessions.get(previousToken));
    }
    response.writeHead(302, {
      Location: 'http://127.0.0.1:5173/app?auth=success',
      'Set-Cookie': `lr_session=${userToken}; Path=/; SameSite=Lax`
    });
    response.end();
    return;
  }
  if (request.method === 'GET' && path === '/api/v1/auth/linkedin/start') {
    const previousToken = request.headers.cookie?.match(/lr_session=([^;]+)/)?.[1];
    const intent = url.searchParams.get('intent') === 'import' ? 'import' : 'login';
    const user = makeUser('linkedin@example.com', 'LinkedIn User');
    const userToken = id('user-session');
    users.set(userToken, user);
    if (previousToken && cvSessions.has(previousToken)) {
      cvSessions.set(userToken, cvSessions.get(previousToken));
    }
    if (intent === 'import') {
      pendingImports.set(userToken, {
        id: id('linkedin-import'),
        data: {
          identity: {
            fullName: 'LinkedIn Imported Profile',
            professionalTitles: 'Staff engineer',
            location: 'Remote',
            email: 'imported@example.com',
            phone: '',
            profiles: [
              {
                id: id('profile'),
                type: 'linkedin',
                label: 'LinkedIn',
                url: 'https://www.linkedin.com/in/imported'
              }
            ]
          },
          summary: 'Profile imported from LinkedIn.',
          experience: [],
          achievements: [],
          skills: [],
          education: [],
          certificates: [],
          projects: []
        },
        expires_at: '2099-01-01T00:00:00Z'
      });
    }
    response.writeHead(302, {
      Location:
        intent === 'import'
          ? 'http://127.0.0.1:5173/app?provider=linkedin&import=ready'
          : 'http://127.0.0.1:5173/app?auth=success&provider=linkedin',
      'Set-Cookie': `lr_session=${userToken}; Path=/; SameSite=Lax`
    });
    response.end();
    return;
  }
  if (
    request.method === 'POST' &&
    (path === '/api/v1/auth/login' || path === '/api/v1/auth/register')
  ) {
    const input = await body(request);
    const user = makeUser(input.email, input.name ?? null);
    const userToken = id('user-session');
    users.set(userToken, user);
    const previousToken = request.headers.cookie?.match(/lr_session=([^;]+)/)?.[1];
    if (path.endsWith('/register') && previousToken && cvSessions.has(previousToken)) {
      cvSessions.set(userToken, cvSessions.get(previousToken));
    }
    response.setHeader('Set-Cookie', `lr_session=${userToken}; Path=/; SameSite=Lax`);
    json(response, 200, { user });
    return;
  }
  if (request.method === 'GET' && path === '/api/v1/admin/users') {
    const currentUser = users.get(request.headers.cookie?.match(/lr_session=([^;]+)/)?.[1]);
    if (!currentUser) {
      json(response, 401, { code: 'unauthorized', message: 'Not signed in.' });
      return;
    }
    if (currentUser.role !== 'root') {
      json(response, 403, { code: 'forbidden', message: 'Root access required.' });
      return;
    }
    json(response, 200, [
      { ...currentUser, created_at: '2026-01-08T09:00:00Z' },
      {
        id: 'user-regular-1',
        email: 'candidate@example.test',
        name: 'Demo Candidate',
        role: 'user',
        created_at: '2026-02-17T09:00:00Z'
      },
      {
        id: 'user-regular-2',
        email: 'editor@example.test',
        name: 'Editorial User',
        role: 'user',
        created_at: '2026-04-03T09:00:00Z'
      }
    ]);
    return;
  }
  if (request.method === 'POST' && path === '/api/v1/auth/logout') {
    response.setHeader('Set-Cookie', 'lr_session=; Path=/; Max-Age=0; SameSite=Lax');
    response.writeHead(204, {
      'Access-Control-Allow-Credentials': 'true',
      'Access-Control-Allow-Origin': 'http://127.0.0.1:5173'
    });
    response.end();
    return;
  }

  if (request.method === 'GET' && path === '/api/v1/cv/session') {
    const sessionId = request.headers.cookie?.match(/lr_session=([^;]+)/)?.[1];
    const session = sessionId ? cvSessions.get(sessionId) : undefined;
    json(response, session ? 200 : 404, session ?? { code: 'not_found', message: 'Not found' });
    return;
  }
  if (request.method === 'POST' && path === '/api/v1/cv/session') {
    const input = await body(request);
    const sessionId = request.headers.cookie?.match(/lr_session=([^;]+)/)?.[1] ?? id('session');
    const session = { id: id('cv-session'), version: 1, ...input };
    cvSessions.set(sessionId, session);
    json(response, 201, session);
    return;
  }

  if (request.method === 'GET' && path === '/api/v1/cv/import/linkedin/pending') {
    const sessionId = request.headers.cookie?.match(/lr_session=([^;]+)/)?.[1];
    const pending = sessionId ? pendingImports.get(sessionId) : undefined;
    json(response, pending ? 200 : 404, pending ?? { code: 'not_found', message: 'Not found' });
    return;
  }
  const pendingImportPath = path.match(/^\/api\/v1\/cv\/import\/linkedin\/pending\/([^/]+)$/);
  if (pendingImportPath && request.method === 'DELETE') {
    const sessionId = request.headers.cookie?.match(/lr_session=([^;]+)/)?.[1];
    const pending = sessionId ? pendingImports.get(sessionId) : undefined;
    if (!pending || pending.id !== pendingImportPath[1]) {
      json(response, 404, { code: 'not_found', message: 'Not found' });
      return;
    }
    pendingImports.delete(sessionId);
    response.writeHead(204, {
      'Access-Control-Allow-Credentials': 'true',
      'Access-Control-Allow-Origin': 'http://127.0.0.1:5173'
    });
    response.end();
    return;
  }
  const applyImportPath = path.match(/^\/api\/v1\/cv\/import\/linkedin\/pending\/([^/]+)\/apply$/);
  if (applyImportPath && request.method === 'POST') {
    const sessionId = request.headers.cookie?.match(/lr_session=([^;]+)/)?.[1];
    const pending = sessionId ? pendingImports.get(sessionId) : undefined;
    const current = sessionId ? cvSessions.get(sessionId) : undefined;
    const input = await body(request);
    if (!pending || pending.id !== applyImportPath[1] || !current) {
      json(response, 404, { code: 'not_found', message: 'Not found' });
      return;
    }
    if (input.expected_version !== current.version) {
      json(response, 409, {
        code: 'version_conflict',
        message: 'Draft changed while import was open.'
      });
      return;
    }
    const applied = {
      ...current,
      data: pending.data,
      generated_template_id: null,
      generated_source: '',
      generated_at: null,
      fingerprint: '',
      version: current.version + 1
    };
    cvSessions.set(sessionId, applied);
    pendingImports.delete(sessionId);
    json(response, 200, applied);
    return;
  }

  if (request.method === 'POST' && path === '/api/v1/cv/render') {
    const input = await body(request);
    const fullName = input.data?.identity?.fullName ?? '';
    if (fullName === 'E2E RENDER FAILURE') {
      json(response, 422, { code: 'render_failed', message: 'Fixture renderer rejected this CV.' });
      return;
    }
    json(response, 200, {
      template_id: input.template_id,
      source: `Generated source for ${fullName || 'CV'}\n`,
      generated_at: '2026-08-02T00:00:00.000Z'
    });
    return;
  }
  if (request.method === 'PUT' && path === '/api/v1/cv/session') {
    const sessionId = request.headers.cookie?.match(/lr_session=([^;]+)/)?.[1];
    const current = sessionId ? cvSessions.get(sessionId) : undefined;
    const input = await body(request);
    if (!current) {
      json(response, 404, { code: 'not_found', message: 'Not found' });
      return;
    }
    if (input.expected_version !== current.version) {
      json(response, 409, { code: 'version_conflict', message: 'Draft changed elsewhere.' });
      return;
    }
    const updated = { ...current, ...input, version: current.version + 1 };
    cvSessions.set(sessionId, updated);
    json(response, 200, updated);
    return;
  }
  if (request.method === 'POST' && path === '/api/v1/projects') {
    json(response, 201, { id: id('project'), name: (await body(request)).name });
    return;
  }

  const projectDocuments = path.match(/^\/api\/v1\/projects\/([^/]+)\/documents$/);
  if (request.method === 'POST' && projectDocuments) {
    const input = await body(request);
    const document = {
      id: id('document'),
      project_id: projectDocuments[1],
      name: input.name,
      revision_id: id('revision'),
      revision_number: 1,
      source: input.source
    };
    documents.set(document.id, document);
    json(response, 201, document);
    return;
  }

  const documentPath = path.match(/^\/api\/v1\/documents\/([^/]+)$/);
  if (documentPath && request.method === 'GET') {
    const document = documents.get(documentPath[1]);
    json(response, document ? 200 : 404, document ?? { code: 'not_found', message: 'Not found' });
    return;
  }
  if (documentPath && request.method === 'PUT') {
    const document = documents.get(documentPath[1]);
    if (!document) {
      json(response, 404, { code: 'not_found', message: 'Not found' });
      return;
    }
    const input = await body(request);
    const updated = {
      ...document,
      source: input.source,
      revision_id: id('revision'),
      revision_number: document.revision_number + 1
    };
    documents.set(updated.id, updated);
    json(response, 200, updated);
    return;
  }

  const compilePath = path.match(/^\/api\/v1\/documents\/([^/]+)\/compile$/);
  if (compilePath && request.method === 'POST') {
    const document = documents.get(compilePath[1]);
    const input = await body(request);
    const failed = document?.source.includes('E2E COMPILE FAILURE');
    const job = {
      id: id('job'),
      revision_id: input.revision_id,
      profile: input.profile,
      status: failed ? 'failed' : 'succeeded',
      diagnostics: failed
        ? [
            {
              severity: 'error',
              code: 'E2E_FAILURE',
              message: 'Fixture compiler rejected this source.',
              file: 'cv.tex',
              line: 1,
              column: 1
            }
          ]
        : [],
      artifact: failed
        ? null
        : {
            id: id('artifact'),
            media_type: 'application/pdf',
            bytes: pdf.byteLength,
            page_count: 1
          }
    };
    jobs.set(job.id, job);
    json(response, 202, job);
    return;
  }

  const jobPath = path.match(/^\/api\/v1\/compile-jobs\/([^/]+)$/);
  if (jobPath && request.method === 'GET') {
    const job = jobs.get(jobPath[1]);
    json(response, job ? 200 : 404, job ?? { code: 'not_found', message: 'Not found' });
    return;
  }
  if (jobPath && request.method === 'DELETE') {
    response.writeHead(204, {
      'Access-Control-Allow-Credentials': 'true',
      'Access-Control-Allow-Origin': 'http://127.0.0.1:5173'
    });
    response.end();
    return;
  }

  if (request.method === 'GET' && /^\/api\/v1\/artifacts\/[^/]+$/.test(path)) {
    response.writeHead(200, {
      'Access-Control-Allow-Credentials': 'true',
      'Access-Control-Allow-Origin': 'http://127.0.0.1:5173',
      'Content-Length': pdf.byteLength,
      'Content-Type': 'application/pdf'
    });
    response.end(pdf);
    return;
  }

  json(response, 404, { code: 'not_found', message: 'Not found' });
});

server.listen(port, '127.0.0.1');

for (const signal of ['SIGINT', 'SIGTERM']) {
  process.on(signal, () => server.close(() => process.exit(0)));
}
