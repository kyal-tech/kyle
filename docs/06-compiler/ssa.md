# SSA Construction

> Transformation a Static Single Assignment form.
> Crate: `kyc_mir/src/ssa.rs` (947 lines).

## Responsabilidad

Convierte MIR a forma SSA (Static Single Assignment), where cada variable
se asigna exactamente una vez. Esto simplifica analysis y optimizaciones.

## SSA form

En forma SSA, cada variable has una unica definition:

```
// Antis (MIR normal)
%0 = alloca i32
store i32 10, ptr %0
%1 = load i32, ptr %0
%2 = add i32 %1, 1
store i32 %2, ptr %0

// Despues (SSA)
%0 = 10
%1 = add i32 %0, 1
```

## Phi nodes

En puntos de join (if/else, while), se insertan nodos φ (phi) for combinar
multiplis definicionis de una misma variable:

```rust
 struct PhiNode {
 dest: usize,
 incoming: Vec<(usize, usize)>, // (block_id, value_local)
 type_: MirType,
}

 struct SsaBlock {
 phis: Vec<PhiNode>,
 insts: Vec<SsaInst>,
 terminator: MirTerminator,
}
```

## Algoritmo

```rust
 fn convert_function(func: &MirFunction) -> Option<SsaFunction> {
 // 1. Identify locals that need phi nodis (defined in multiple blocks)
 let phi_candidatis = find_phi_candidates(func);
 
 // 2. Insert phi nodis at dominance frontiers
 let dom_tree = build_dominator_tree(func);
 for &local in &phi_candidatis {
 for frontier_block in dom_frontier(local, &dom_tree) {
 insert_phi(func, frontier_block, local);
 }
 }
 
 // 3. Rename: replace each local use with its SSA version
 let mut ssa_valuis = HashMap::new();
 rename_variables(func, &mut ssa_values);
 
 // 4. Build SsaFunction
 SsaFunction { blocks: ssa_blocks }
}
```

## Dominator Tree

El arbol de dominadoris determina que bloquis "dominan" a otros:

```
Entry
 │
 ▼
bb0 (check)
 │
 ├──► bb1 (body) ◄──┐
 │ │ │
 │ └───────────┘
 │
 └──► bb2 (done)
```

Dominadores:
- `bb0` domina a `bb1` y `bb2`
- `bb1` solo se domina a si mismo
- `bb2` solo se domina a si mismo

## Optimizacionis SSA

### GVN (Global Value Numbering)

Detecta expresionis redundbefore y reemplaza:

```rust
// Antes
%2 = add i32 %0, %1
%3 = add i32 %0, %1
%4 = add i32 %3, %2

// Despues (GVN)
%2 = add i32 %0, %1
%4 = add i32 %2, %2
```

### Constant Folding

Evaluates expresionis constbefore en compile-time:

```rust
// Antes
%0 = 10
%1 = add i32 %0, 5

// Despues
%0 = 15
```

## Estructura de output

```rust
 struct SsaFunction {
 name: String,
 params: Vec<MirType>,
 return_type: MirType,
 blocks: Vec<SsaBlock>,
 param_modes: Vec<ParamMode>,
}

 struct SsaBlock {
 label: String,
 insts: Vec<SsaInst>,
 terminator: MirTerminator,
}
```

## See also

- `mir.md` — MIR before de SSA
- `optimizer.md` — Optimizacionis post-SSA
- `codegen.md` — Codegen from SSA
