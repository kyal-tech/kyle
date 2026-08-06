# Contributing to Kyle — Guía de Colaboración

> Documento oficial de referencia para contribuir al repositorio Kyle.
> Léelo completo antes de tu primer commit. Es obligatorio para todos los colaboradores.

---

## Tabla de contenidos

1. [Modelo de ramas](#1-modelo-de-ramas)
2. [Convenciones de commits](#2-convenciones-de-commits)
3. [Flujo de trabajo con Pull Requests](#3-flujo-de-trabajo-con-pull-requests)
4. [Revisión de código](#4-revisión-de-código)
5. [Issues, labels y milestones](#5-issues-labels-y-milestones)
6. [Versionado semántico y releases](#6-versionado-semántico-y-releases)
7. [Protección de ramas](#7-protección-de-ramas)
8. [Mantener tu rama al día](#8-mantener-tu-rama-al-día)
9. [Estructura del repositorio](#9-estructura-del-repositorio)
10. [Verificación local (build y tests)](#10-verificación-local-build-y-tests)
11. [Hotfixes y emergencias](#11-hotfixes-y-emergencias)
12. [Guía rápida para nuevos colaboradores](#12-guía-rápida-para-nuevos-colaboradores)
13. [Escalabilidad: cómo crecer sin romper nada](#13-escalabilidad-cómo-crecer-sin-romper-nada)
14. [Comunicación y código de conducta](#14-comunicación-y-código-de-conducta)

---

## 1. Modelo de ramas

Kyle usa un **Git Flow simplificado**, pensado para equipos pequeños que quieren
escalar sin fricción.

```
main                (producción — siempre estable, protegida)
 └── develop        (integración — donde se junta el trabajo en curso)
      ├── feature/xxx     ← nueva funcionalidad
      ├── fix/xxx         ← corrección de bug
      ├── docs/xxx        ← documentación
      ├── chore/xxx       ← tareas de mantenimiento (build, deps, refactor)
      ├── test/xxx        ← solo tests
      └── perf/xxx        ← optimizaciones
           └── release/vX.Y.Z  ← solo se crea al preparar una versión
```

### Reglas fundamentales

| Rama | Quién puede escribir | Qué llega ahí |
|------|---------------------|---------------|
| `main` | **Nadie directamente.** Solo vía PR aprobado y mergeado | Código 100% estable, versionado |
| `develop` | Vía PR (preferido) o con criterio de mantenimiento | Funcionalidades listas para integrar |
| `feature/*`, `fix/*`, etc. | Cualquier colaborador, en su propia rama | Trabajo en curso |

### Reglas de la rama

- **Nunca** se hace commit directamente sobre `main`. Está protegida por GitHub.
- Toda rama nueva sale desde `develop` (o desde `main` para hotfixes).
- Naming en minúsculas con `-`: `feature/std-net`, `fix/c_char-linux`, `docs/workflow`.
- Una rama = **un objetivo**. Si se mezclan dos ideas, se separan en dos ramas/PRs.
- Las ramas de trabajo deben eliminarse después de mergear.

---

## 2. Convenciones de commits

Usamos **Conventional Commits**. Formato:

```
<tipo>(<scope>): <descripción>

[ cuerpo opcional ]

[ BREAKING CHANGE: descripción o pie opcional ]
```

### Tipos permitidos

| Tipo | Cuándo usarlo | Ejemplo |
|------|---------------|---------|
| `feat` | Nueva funcionalidad | `feat(std/net): add TCP server bind` |
| `fix` | Corrección de bug | `fix(runtime): c_char mismatch on Linux ARM` |
| `docs` | Solo documentación | `docs: add contributing guide` |
| `style` | Formato, sin cambio de lógica | `style: apply rustfmt` |
| `refactor` | Reestructurar sin cambiar comportamiento | `refactor(mir): simplify SSA builder` |
| `perf` | Optimización de rendimiento | `perf(backend): reduce LLVM module copies` |
| `test` | Añadir/corregir tests | `test(parser): cover range syntax` |
| `build` | Cambios en build/deps | `build: bump llvm-sys to 18` |
| `ci` | Cambios en pipelines | `ci: add ARM runner to matrix` |
| `chore` | Mantenimiento misceláneo | `chore: update Cargo.lock` |
| `revert` | Revertir un commit | `revert: feat(std/time) broken TZ` |

### Reglas

- El `scope` (opcional) indica el módulo afectado: `core`, `frontend`, `hir`,
  `semantic`, `mir`, `backend`, `runtime`, `cli`, `std/<mod>`, `ui`, `packages/<name>`.
- La descripción en **imperativo, minúscula, sin punto final**.
- Si el cambio rompe la compatibilidad del lenguaje (cambia sintaxis, tipos o
  semántica), añade `!` o la línea `BREAKING CHANGE:`:
  - `feat(std/str)!: rename find_idx to index_of`
- Máximo ~72 caracteres en la línea de asunto.
- Un commit debe ser **atómico**: un solo cambio lógico, que compile y pase tests.

---

## 3. Flujo de trabajo con Pull Requests

Todo cambio a `main`/`develop` pasa por un **Pull Request**. Flujo completo:

1. **Sincroniza** tu fork/clon:
   ```bash
   git fetch origin
   git checkout develop && git pull origin develop
   ```
2. **Crea tu rama** con nombre descriptivo:
   ```bash
   git checkout -b feature/descripcion
   ```
3. **Implementa** con commits atómicos y convencionales (ver §2).
4. **Mantén tu rama al día** con `develop` (ver §8).
5. **Verifica en local** (ver §10): build, tests y clippy.
6. **Sube tu rama**:
   ```bash
   git push -u origin feature/descripcion
   ```
7. **Abre el PR** desde GitHub usando la plantilla. Apunta contra `develop`
   (o `main` solo para hotfixes).
8. **Atiende los comentarios** de la revisión. Responde a cada comentario.
9. Cuando el CI esté **verde** y tengas **1 aprobación**, se mergea.

### Reglas del PR

- Título del PR en formato convencional: `feat(std/net): add server bind`.
- Mínimo: describe **qué** haces y **por qué**. Adjunta **cómo lo probaste**.
- Un PR = un objetivo. Los PRs pequeños se revisan y mergean más rápido.
- Referencia issues con `Closes #123`.
- **No** mergees tu propio PR si el proyecto tiene revisión cruzada (ver §4).

---

## 4. Revisión de código

La revisión es obligatoria y es **la defensa de calidad del proyecto**.

### Roles

- **Author**: quien abre el PR.
- **Reviewer**: quien revisa (1 aprobación requerida por la protección de `main`
  una vez el equipo tenga revisión cruzada; hoy el requisito es `0`).

### Qué revisar

- Correctitud (compila, tests pasan, lógica es correcta).
- Estilo y consistencia con el resto del código.
- Que respete convenciones (commits, nombres, scope).
- Rendimiento cuando aplique (compilador/MIR/backend).
- **Nada de secrets**: tokens, passwords, claves. Nunca se commitean.

### Reglas

- Mantén los PRs pequeños (< ~400 líneas idealmente; los grandes se dividen).
- Usa *suggestions* de GitHub para cambios concretos.
- Si el CI falla, el PR **no** se mergea.
- Aprobación = el PR puede mergearse; no necesariamente está terminado.
- El autor debe dar las gracias/explicar cuando discuta un comentario.

---

## 5. Issues, labels y milestones

### Labels estándar

| Label | Uso |
|-------|-----|
| `bug` | Comportamiento incorrecto documentado |
| `enhancement` | Nueva funcionalidad propuesta |
| `compiler` / `runtime` / `std` / `ui` / `packages` | Área del código |
| `good first issue` | Tareas fáciles para nuevos colaboradores |
| `help wanted` | Se busca voluntario |
| `needs triage` | Sin clasificar aún |
| `blocked` | Depende de otra tarea |
| `P1` / `P2` / `P3` | Prioridad (urgente / normal / baja) |

### Buenas prácticas

- Antes de crear un issue, **busca** si ya existe (evita duplicados).
- Usa las **plantillas** de issue (bug report / feature request).
- Los bugs siguen: descripción → pasos para reproducir → resultado esperado →
  resultado real → entorno (OS, versión).
- Usa **milestones** por versión (`v0.10.0`) y asocia los issues/PRs.
- Un issue cerrado que reaparece se reabre, no se crea uno nuevo.

---

## 6. Versionado semántico y releases

Kyle sigue **SemVer**: `MAJOR.MINOR.PATCH`.

| Incremento | Cuándo |
|-----------|--------|
| MAJOR | Breaking change en el lenguaje o API pública |
| MINOR | Nueva funcionalidad compatible |
| PATCH | Bug fixes compatibles |

### Flujo de release

1. Se crea `release/vX.Y.Z` desde `develop`.
2. Se hacen los ajustes finales (changelog, bump de versión).
3. Se taggea `vX.Y.Z` y se pusha el tag:
   ```bash
   git tag -a v0.10.0 -m "v0.10.0"
   git push origin v0.10.0
   ```
4. El workflow de GitHub **Release** (`release.yml`) compila los binarios para
   todas las plataformas y sube los artefactos automáticamente.
5. `release/vX.Y.Z` se mergea a `main` y se vuelve a `develop`.

### Reglas

- El tag se hace sobre un commit que pasó CI completo.
- Los releases son inmutables: no se reescribe la historia de un tag publicado.
- `main` solo recibe versiones reales (via `release/*` u hotfix).

---

## 7. Protección de ramas

`main` está protegida en GitHub. Esto significa:

- **Push directo bloqueado**: nadie puede hacer `git push origin main`.
- **PR obligatorio**: todo cambio entra vía Pull Request.
- **Aprobación requerida**: hoy `0` (colaborador único); se subirá a `>=1` cuando
  haya revisión cruzada de equipo.
- **CI obligatorio**: el status check debe estar verde antes de mergear.
- **Historia limpia**: force-push y eliminación de la rama bloqueados.

Si necesitas cambiar la protección, hazlo desde *Settings → Branches → Branch
protection rules* (o *Rulesets*). No lo hagas en cada PR "por comodidad".

---

## 8. Mantener tu rama al día

Cuando `develop` avanza y tu feature branch se queda atrás:

### Opción A — Merge (recomendada para PRs abiertos)

```bash
git fetch origin
git merge origin/develop
```

### Opción B — Rebase (historia más limpia, solo en ramas locales)

```bash
git fetch origin
git rebase origin/develop
```

> ⚠️ **Nunca hagas force-push sobre ramas compartidas** (`main`, `develop`,
> `release/*`). Sobre tu feature branch local sí puedes rebasar, pero si ya la
> subiste, usa `--force-with-lease` con cuidado.

### Resolver conflictos

1. Los conflictos se resuelven en tu editor.
2. Marca los archivos como resueltos: `git add <archivo>`.
3. Continúa: `git rebase --continue` o `git merge --continue`.
4. Vuelve a verificar con build + tests (§10).

---

## 9. Estructura del repositorio

```
crates/            ← Rust: compilador y herramientas
  kyc_frontend/    → Lexer + parser
  kyc_hir/         → HIR
  kyc_semantic/    → Type checker + borrow checker
  kyc_mir/         → MIR
  kyc_backend/     → LLVM codegen
  kyc_driver/      → Pipeline
  kyc_cli/         → Binario `ky`
  kyc_runtime/     → Runtime (190+ extern "C")
  kyc_tools/       → LSP, formatter, package manager
packages/          → Paquetes instalables (`ky add`)
docs/              → Docs del lenguaje, stdlib, planes
tests/             → Tests de sintaxis y packages
examples/          → Programas .ky / .kyx
scripts/           → install.sh, dev-install.sh
.github/workflows/ → CI + Release
```

Documentación clave: `AGENTS.md`, `BUILD.md`, `PACKAGES.md`.

---

## 10. Verificación local (build y tests)

**Antes de abrir un PR, siempre corre estas verificaciones:**

```bash
# Build
cargo build --release --bin ky

# Tests de workspace (excluye wasm que no aplica en CI)
cargo test --workspace --exclude kyc_runtime_wasm

# Tests de sintaxis
for f in tests/syntax/*.ky; do ky run "$f"; done
for f in tests/std_*.ky; do ky run "$f"; done

# Clippy (sin warnings)
cargo clippy --workspace -- -D warnings

# Tests de un paquete
ky test packages/<name>/tests/
```

**Regla**: si no compila o fallan tests, el PR no se abre. Punto.

---

## 11. Hotfixes y emergencias

Un hotfix corrige un bug crítico en producción (en `main`/versión publicada).

1. Crea la rama desde **`main`** (no desde `develop`):
   ```bash
   git checkout main && git pull
   git checkout -b fix/<bug-critico>
   ```
2. Corrige, verifica, abre PR **contra `main`**.
3. Tras mergear el PR, integra el fix también en `develop` (merge o cherry-pick).

Los hotfixes son la única excepción al "todo entra por `develop`".

---

## 12. Guía rápida para nuevos colaboradores

1. Lee `README.md`, `BUILD.md` y esta guía.
2. Busca issues etiquetados `good first issue`.
3. Comenta en el issue que lo vas a tomar (evita trabajo duplicado).
4. Fork del repo → clona → crea rama `feature/...`.
5. Sigue el flujo de §3 y verifica con §10.
6. Abre el PR y atiende la revisión.

### Checklist previo al PR

- [ ] Compila en local (`cargo build --release --bin ky`)
- [ ] Tests pasan (workspace + sintaxis + std)
- [ ] Clippy sin errores
- [ ] Commit con formato convencional
- [ ] Rama nombrada correctamente
- [ ] PR describe qué/por qué/cómo-probé
- [ ] Sin secrets ni archivos de build (`.gitignore` respetado)

---

## 13. Escalabilidad: cómo crecer sin romper nada

Estas reglas están pensadas para que el proyecto escale de 1 a N colaboradores:

- **Ramas por feature** + PRs pequeños: permite trabajar en paralelo sin colisión.
- **Protección de `main`**: nada se rompe en producción; la integración se
  prueba por CI en cada PR.
- **Conventional commits**: genera changelogs y releases automáticos legibles.
- **Milestones**: agrupan trabajo por versión y dan visibilidad del roadmap.
- **Merge queue** (cuando el equipo crezca): ordena y re-testea los merges
  automáticamente.
- **CODEOWNERS**: cada área tiene dueño responsable de su revisión.
- **Templates**: estandarizan issues y PRs, reduciendo el ruido.
- Si un área crece mucho, los monorepo crates se pueden separar o añadir
  *CODEOWNERS* por directorio. El modelo actual ya lo permite.

---

## 14. Comunicación y código de conducta

- Sé respetuoso: la crítica es al código, nunca a la persona.
- Explica con claridad; asume que el otro colaborador puede no tener contexto.
- Los cambios grandes **se discuten antes de codificar** (issue + plan).
- Pregunta en un issue si no estás seguro; no adivines.
- No se aceptan PRs que introduzcan secrets, binarios grandes o dependencias
  sin justificar.

---

*Última actualización: v0.9.1 — si algo cambia en el flujo, actualiza este documento en el mismo PR.*
