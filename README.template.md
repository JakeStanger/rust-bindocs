# Rust Bindocs

A tool to assist writing documentation for Rust binaries.

---

Rustdoc is a brilliant tool, but it's only really suited to libraries.
Documenting applications can be a pain,
because you often need to reference types from the code,
but don't want to include all the internal information or require end-users to look at code docs.
The alternative is writing everything by hand, but keeping that in sync with code changes can be a pain
and it is easy to make mistakes.

This tool is designed to simplify that process by providing a basic templating engine
that can generate documentation straight from your source code, optimised for end-users,
and inlined in your existing docs.

Existing Rustdoc comments are fully supported, 
and the contained markdown is seemingly integrated into your document.

> ⚠️ The tool is currently in its infancy. 
> Only Markdown output is supported, and expect bugs.

## Installation

`cargo install bindocs`

[crate](https://crates.io/bindocs)

## Usage

The crate includes a CLI.
Point it at the root of a crate, optionally specify the documentation input/output directories,
and your docs will be rendered out.

<% Args { depth = 2 } %>

---

Inside your input markdown, use `<%  template_blocks  %>` to denote where types should automatically be injected.
You can inject any struct or enum owned by your crate.

For example, if you have a `config` module containing a `MyConfig` struct:

```rust
struct MyConfig {
    /// Enables foo mode.
    foo: bool,
    
    /// Specifies the `bar` value to use.
    /// 
    /// # Example
    /// 
    /// ```json
    /// { "bar" = "baz" }
    /// ```
    bar: String,
}
```

You can inject it into your documentation as follows:

```markdown
# My docs page

Lorem ipsum dolar sit amet.

<%  config::AppConfig  %>

Ornare lectus sit amet est placerat in egestas.
```

This will produce an output similar to the below:

````markdown
## MyConfig

### foo

> Type: `bool`

Enables foo mode.

### bar

> Type: `String`

Specifies the `bar` value to use.

#### Example

```json
{ "bar": "baz" }
```
````

> ✅ If the type is uniquely named within your project, 
> you can omit the path (ie just `AppConfig`) and bindocs will resolve it still.

### Configuring injections

Each injection can be individually configured using [Corn](https://github.com/jakestanger/corn)
after the path to the type, before the closing brace.

For example, to change the heading depth:

```markdown
<%  config::AppConfig { depth = 3 }  %>
```

#### Injection replace options

<% ReplaceOptions { header = false depth = 3 } %>

## Contributing

Contributions are welcome!

If you find any issues, please open a bug report.

If you have a feature request, please open an issue first to allow it to be discussed.
More options and render formats are more than likely to be accepted.