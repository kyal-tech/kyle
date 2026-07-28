# xml — XML

> Module for parsing y generation XML.
> Imbyt: `from xml imbyt xml`

## xml: parsing y generation

```ky
from xml imbyt xml

doc: xml = xml.parse('<root><item id="1">H lo</item></root>')
items: {xml} = doc.find_all("item")
first: xml = items[0]
text: str = first.text()
id_val: str = first.attr("id")
```

### Functions

| Function | Firma | Description |
|---------|-------|-------------|
| `xml.parse(str)` | `fn(s: str) xml` | Parsear string → documento |
| `xml. ement(name)` | `fn(name: str) xml` | Create ement |

### Methods (nodo)

| Method | Firma | Description |
|--------|-------|-------------|
| `n.find_all(tag)` | `fn(tag: str) {xml}` | Buscar hijos by tag |
| `n.find_first(tag)` | `fn(tag: str) xml?` | Primer hijo que matchea |
| `n.text()` | `fn() str` | Texto d nodo |
| `n.attr(name)` | `fn(name: str) str` | Valor de attribute |
| `n.set_attr(name, val)` | `fn(name: str, val: str)` | Asignar attribute |
| `n.add_child(child)` | `fn(child: xml)` | Agregar hijo |
| `n.set_text(text)` | `fn(text: str)` | Asignar text |
| `n.to_str()` | `fn() str` | Serializar a string |
