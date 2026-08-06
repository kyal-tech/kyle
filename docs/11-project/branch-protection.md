# Protección de ramas en GitHub — Guía y Script Reutilizable

Cómo proteger ramas en cualquier repo de GitHub, ya sea por **terminal** o desde
la **web**. El script `scripts/protect-branch.sh` automatiza la configuración
por API y sirve para cualquiera de tus proyectos.

---

## 1. Script reutilizable (terminal)

```bash
# PROTEGER una rama (PR obligatorio + 1 aprobación + CI verde + sin force-push)
./scripts/protect-branch.sh OWNER REPO BRANCH

# Ejemplos
./scripts/protect-branch.sh kyal-tech kyle main
./scripts/protect-branch.sh kyal-tech kyle develop
./scripts/protect-branch.sh mi-usuario mi-app main

# QUITAR la protección
./scripts/protect-branch.sh OWNER REPO BRANCH --remove
```

### Configuración

| Variable | Qué hace | Default |
|----------|----------|---------|
| `GITHUB_TOKEN` | Token de acceso (si no usas keychain ni gh) | auto-detecta |
| `REQUIRED_CHECKS` | Checks de CI a exigir, separados por `\|` | los 3 del CI de Kyle |
| `APPROVALS` | Aprobaciones de revisión requeridas (`0` solo; `>=1` con equipo) | `0` |

```bash
# Usar tu propio CI (los nombres exactos salen del run de GitHub)
REQUIRED_CHECKS="CI / build|CI / test (ubuntu-24.04)" \
  ./scripts/protect-branch.sh mi-usuario mi-app main

# Con equipo: exigir 1 revisión aprobada
APPROVALS=1 ./scripts/protect-branch.sh mi-usuario mi-app main
```

### Autenticación (orden de prioridad)
1. `GITHUB_TOKEN` exportado (recomendado).
2. `gh` CLI (`gh auth login`).
3. Keychain de macOS (`git credential fill`).

> Para sacar los nombres exactos de tus checks:
> `gh api repos/OWNER/REPO/commits/main/check-runs --jq '.check_runs[].name'`

### Qué protege el script
- ☑ Require PR (obligatorio siempre)
- ☑ Require status checks (strict — rama al día)
- ☑ Aplica a admins (`enforce_admins`)
- ☑ Bloquea force-push y eliminación de la rama
- Aprobaciones: `0` por defecto (colaborador único). Con equipo: `APPROVALS=1`

> **Nota GitHub**: la aprobación del propio autor de un PR no cuenta. Con
> `APPROVALS=1` y un solo colaborador, los PRs quedarán esperando a un segundo
> revisor. Por eso el default es `0` para equipos pequeños, y se sube a `>=1`
> cuando haya revisión cruzada real.

---

## 2. Por terminal con `gh` (sin script)

```bash
gh api --method PUT /repos/USER/REPO/branches/main/protection \
  -F required_status_checks[strict]=true \
  -F required_status_checks[contexts][]=CI / test (ubuntu-24.04) \
  -F required_pull_request_reviews[required_approving_review_count]=1 \
  -F enforce_admins=true \
  -F allow_force_pushes=false \
  -F allow_deletions=false

# Quitar
gh api --method DELETE /repos/USER/REPO/branches/main/protection
```

---

## 3. Desde la web de GitHub (sin terminal)

1. Repo → **Settings** → **Branches** → **Add branch protection rule**.
2. *Branch name pattern*: `main` (o `*` para una regla genérica).
3. Marcar:
   - **Require a pull request before merging** → *Require approvals* = 1
   - **Require status checks to pass before merging** → elegir los checks → *Require branches to be up to date*
   - **Do not allow bypassing the above settings**
   - **Block force pushes** + **Block deletions**
4. **Create**. Repetir para `develop` si aplica.

> **Rulesets** (Settings → Rules → Rulesets) es la alternativa moderna con más
> control (por ejemplo, por usuario o por patrón de rama). Tiene el mismo efecto.

---

## 4. Equivalencia

| Acción | Web | Terminal |
|--------|-----|----------|
| Proteger rama | Settings → Branches → rule | `./scripts/protect-branch.sh O R B` |
| Quitar | Settings → Branches → Delete rule | `./scripts/protect-branch.sh O R B --remove` |
| Ver regla actual | Settings → Branches | `gh api repos/O/R/branches/B/protection` |

La web es visual y suficiente para una sola vez; el script sirve para replicar
la misma configuración en muchos repos en segundos.
