# Vercel deployment runbook

Agentbriefer documentation is a static Docusaurus application and can be hosted by Vercel independently
from the main Agentbriefer website.

## Create the Vercel project

1. Import `dexterhere/agentbriefer` as a new Vercel project.
2. Set **Root Directory** to `docs-site`.
3. Keep the framework preset as **Other** or let Vercel use `vercel.json`.
4. Confirm the install command is `npm install`, build command is `npm run build`, and output directory
   is `build`.
5. Choose the production branch used for stable documentation and enable pull-request previews.
6. Deploy and verify navigation, version switching, search, `/sitemap.xml`, `/llms.txt`, and
   `/llms-full.txt`.

No server runtime or environment secrets are required for the current site.

## Connect the custom domain

1. Add `docs.agentbriefer.com` to the documentation Vercel project.
2. In the DNS provider for `agentbriefer.com`, add the record Vercel requests.
3. Wait for Vercel to verify DNS and issue TLS.
4. Confirm `https://docs.agentbriefer.com/` is canonical and the main website links to it.

Do not attach the docs domain to the existing website project; it belongs to this separate Vercel
project even though both are managed under the same account.

## Production verification

```bash
curl -I https://docs.agentbriefer.com/
curl -I https://docs.agentbriefer.com/llms.txt
curl -I https://docs.agentbriefer.com/sitemap.xml
```

Also test one deep documentation URL, local search, the stable/Next version menu, mobile navigation,
and a pull-request preview. Roll back through Vercel deployments if a production build is healthy but
the published content is incorrect.

