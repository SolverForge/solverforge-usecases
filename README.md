# Sync uc-* → Hugging Face Spaces

GitHub Actions workflow that automatically publishes each `uc-xx`
subfolder to the Hugging Face Space `<org>/solverforge-xx` of the corresponding name,
using `git subtree split` to **preserve commit history**.

---

## Expected Repository Structure

```
monorepo/
├── .github/
│   └── workflows/
│       └── sync-hf-spaces.yml
├── uc-01/
│   ├── app.py
│   └── README.md          ← HF card (optional)
├── uc-02/
│   └── ...
└── uc-xx/
    └── ...
```

> **Note**: Local directories are named `uc-*` but are published to Hugging Face Spaces as `solverforge-*`.
> For example: `uc-01` → `huggingface.co/spaces/<org>/solverforge-01`

---

## Configuration (One-Time Setup)

### 1. Secret — Hugging Face Token

Go to **Settings → Secrets and variables → Actions**, tab **Secrets**, then **New repository secret**:

| Name | Value |
|------|-------|
| `HF_TOKEN` | HF token with **write** role on your Spaces (`hf_…`) |

To generate the token:
1. Go to https://huggingface.co/settings/tokens
2. Click **New token** → type **Write** → copy the `hf_…` value
3. Paste it as the secret value in GitHub

> Secrets are encrypted and never displayed in plain text. Their value is automatically
> masked (`***`) in workflow logs and accessible via `${{ secrets.HF_TOKEN }}`.

### 2. Variable — HF Organization

Go to **Settings → Secrets and variables → Actions**, tab **Variables**, then **New repository variable**:

| Name | Value |
|------|-------|
| `HF_ORGANIZATION` | Your HF username or organization (e.g., `myorg`) |

### 3. Create the Spaces in Advance

Each Space `<HF_ORGANIZATION>/solverforge-xx` must exist on HF **before**
the first push (the workflow does not create them).

> **Important**: The HF Space name must use the `solverforge-*` prefix, not `uc-*`.
> For `uc-01` locally, create the Space as `solverforge-01` on HF.

Create them via https://huggingface.co/new-space, choosing the appropriate SDK
(Gradio, Streamlit, Static…).

---

## Triggers

| Event | Behavior |
|-------|----------|
| `push` to `main` affecting `uc-*/**` | Sync **only** modified folders |
| `workflow_dispatch` → target = `all` | Sync **all** `uc-*` folders |
| `workflow_dispatch` → target = `uc-02` | Sync **only** this folder |

---

## How It Works

The workflow automatically transforms local `uc-*` directory names to `solverforge-*`
Space names when pushing to Hugging Face.

```
git subtree split --prefix uc-xx -b <temp-branch>
git push https://…@huggingface.co/spaces/<org>/solverforge-xx <temp-branch>:main
```

`git subtree split` replays only commits that touch the subfolder,
reconstructing a clean history on the HF Space side.

---

## Troubleshooting

| Symptom | Likely Cause | Solution |
|---------|--------------|----------|
| `remote: Invalid credentials` | `HF_TOKEN` expired or missing | Regenerate the HF token |
| `Repository not found` | HF Space does not exist | Create the Space manually (remember: use `solverforge-*` naming) |
| `fatal: prefix '...' not found` | Folder missing from repo | Check the exact name (should be `uc-*`) |
| Push rejected (non-fast-forward) | Diverging histories | `--force` normally handles this case |