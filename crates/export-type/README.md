# export-type

A Rust proc-macro crate for automatically generating TypeScript type definitions from Rust structs and enums.

## Features

- Export Rust structs and enums to TypeScript
- Support for generics
- Field renaming with `rename` and `rename_all` attributes
- Support for common Rust types:
  - Basic types (numbers, strings, booleans)
  - Collections (Vec, HashMap)
  - Optional values (Option<T>)
  - Custom types
- Generates a single TypeScript file with all types

## Installation

Add this to your `Cargo.toml`:

```toml
export-type = { version = "0.1.1", optional = true }
```

## Usage

Use the `#[export_type]` attribute on structs and enums.

Example:

```rust
#[derive(ExportType)]
#[export_type(rename_all = "camelCase", path = "frontend/src/types")]
struct MyStruct {
    field: String,
}

#[derive(ExportType)]
#[export_type(rename_all = "camelCase", path = "frontend/src/types")]
enum MyEnum {
    Variant1,
    Variant2,
}
```

And you'll get a `index.ts` file in the specified path with the following contents:

```typescript
export type MyStruct = {
    field: string;
};

export type MyEnum = "Variant1" | "Variant2";
```
