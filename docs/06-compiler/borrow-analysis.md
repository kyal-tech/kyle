# Borrow Analysis

> Analysis de ownership: verifica use-after-move, one-mut-XOR-many-immut, e inserta `ky_free`.
> Crate: `kyc_mir/src/borrow_analysis.rs` (900 lines).

## Responsabilidad

El borrow analysis opera about MIR para:

1. **Insertar `ky_free`** for valueis Move (str, list, dict, array) al final de cada scope
2. **Detectar use-after-move**: `y = x; println(x)` → error
3. **Validar aliasing**: `r1 = ^&x; r2 = &x` → error (one mut XOR many immut)
4. **Rastrear borrows**: mantener consistencia between referencias

## Classification de types

```rust
 fn is_move_type(t: &MirType) -> bool {
 match t {
 MirType::Str => true,
 MirType::List(_) => true,
 MirType::Dict(_, _) => true,
 // Array y Struct NO are heap-allocated — are value typis en stack.
 // Struct fields (str, list) se trackean independientemente.
 MirType::Struct(_, _) => false,
 _ => false,
 }
}
```

- **Copy**: i8-u64, f32-f64, bool, char, void, ptr — se copian, no se rastrean
- **Move**: str, {T}, {K:V}, [T, N], structs — ownership tracking

## Algoritmo

### 1. Compute alive_in (dataflow)

Computation forward dataflow: determina que valueis Move are "vivos" en cada punto
del program. Usa intersection en join points.

```rust
fn compute_alive_in(&self, func, move_locals, local_types, param_locals, func_map) -> Vec<BTreeSet<usize>> {
 // Initial: params are alive at entry
 // Propagation: forward dataflow with intersection at joins
 // Result: alive_in[block_idx] = set of alive locals entering each block
}
```

### 2. Process instructions (error checking)

Cada instruccion MIR se procesa for verify uso correcto de ownership:

```rust
fn process_inst(&mut self, inst: &MirInst, alive, ...) {
 match inst {
 MirInst::Store { dest, value } => {
 // MOVE: source is consumed
 if let MirValue::Local(src) = value {
 self.check_alive(src, alive, "move");
 alive.remove(src);
 // Also remove original if src was a Load alias
 if let Some(&orig) = load_map.get(src) {
 alive.remove(&orig);
 }
 }
 }
 MirInst::Call { name, args, .. } => {
 // Check param modis from func_map
 let modis = func_map.get(name);
 for (i, arg) in args.iter().enumerate() {
 match mode {
 Borrow → // Immutable borrow: check mutex
 MutableBorrow → // Mutable borrow: check mutex
 Move → // Move: check alive, then remove
 }
 }
 }
 MirInst::Load { dest, src } => {
 // Read access - check alive
 self.check_alive(src, alive, "read");
 alive.insert(dest);
 }
 }
}
```

### 3. Insert ky_free

Al finalizar analysis, se insertan llamadas a `ky_free` for valueis Move
que salen de scope:

```rust
for local in &locals_to_free {
 match local_types.get(local) {
 Some(List(_)) => "ky_list_free",
 Some(Dict(_, _)) => "ky_dict_free",
 _ => "ky_free",
 };
 block.insts.push(Call { name: free_name, args: [local] });
}
```

## Mecanismo de deteccion

### Use-after-move

El analysis manhas un conjunto `alive` de localis Move que aun conhave valuees
validos. Cuando un value se mueve (Store, Call with Move param), se elimina del conjunto.
Cualquier Load posterior from ese local disfor un error.

```ky
s = "hola" # s is alive
t = s # Store: s se mueve → s eliminado de alive
println(s) # Load: s no is en alive → ERROR
```

### Aliasing (one mut XOR many immut)

```rust
#[derive(Clone, Copy, PartialEq)]
enum BorrowState {
 NotBorrowed,
 ImmBorrowed(u32), // count of active immutable borrows
 MutBorrowed,
}
```

```ky
read(&x) # ImmBorrowed(1)
read(&x) # ImmBorrowed(2)
append(^&x) # ImmBorrowed → MutBorrowed → ERROR

append(^&x) # MutBorrowed
read(&x) # MutBorrowed → ImmBorrowed → ERROR
```

## Load Map

Para rastrear alias, se construye un `load_map`: mapea destinos de Load a sus sources.

```rust
let mut load_map: HashMap<usize, usize> = HashMap::new();
for block in &func.basic_blocks {
 for inst in &block.insts {
 if let MirInst::Load { dest, src } = inst {
 if local_types.get(dest).map_or(false, |t| is_move_type(t)) {
 load_map.insert(*dest, *src);
 }
 }
 }
}
```

Cuando un temp que fue cargado from una variable se mueve, tambien se marca 
variable original as movida.

## See also

- `mir.md` — MIR about que opera borrow analysis
- `ssa.md` — Transformation SSA posterior
- `03-language/memory/` — Reglas de ownership del language
