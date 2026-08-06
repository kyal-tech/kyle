---
name: Pull Request
about: Propuesta de cambio para Kyle
title: ""
labels: []
assignees: ""
---

## Descripción

<!-- Qué hace este PR y por qué. Conecta con un issue si existe: Closes #123 -->

## Tipo de cambio

- [ ] `feat` — nueva funcionalidad
- [ ] `fix` — corrección de bug
- [ ] `docs` — documentación
- [ ] `refactor` — reestructuración sin cambio de comportamiento
- [ ] `perf` — optimización
- [ ] `test` — tests
- [ ] `build` / `ci` — build o pipelines
- [ ] `chore` — mantenimiento
- [ ] BREAKING CHANGE (cambia sintaxis/API pública del lenguaje)

## Alcance (scope)

<!-- Área afectada: core, frontend, hir, semantic, mir, backend, runtime, cli, std/<mod>, ui, packages/<name> -->

## Cómo lo probé

<!-- Comandos ejecutados y resultados. Ej: cargo test -p kyc_runtime; ky run tests/std_*.ky -->

## Checklist

- [ ] Compila: `cargo build --release --bin ky`
- [ ] Tests workspace: `cargo test --workspace --exclude kyc_runtime_wasm`
- [ ] Tests de sintaxis y std
- [ ] Clippy sin errores: `cargo clippy --workspace -- -D warnings`
- [ ] Commits con formato convencional
- [ ] Sin secrets ni archivos de build

## Screenshots / evidencia (opcional)

<!-- Si aplica, pega salida o captura -->
