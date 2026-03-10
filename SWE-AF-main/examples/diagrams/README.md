# Diagrams

A command-line tool for creating architecture diagrams from a simple DSL (Domain-Specific Language). Generate production-quality SVG diagrams or preview them as ASCII art in your terminal.

## Features

- 📝 **Simple DSL syntax** for defining nodes and connections
- 🎨 **SVG output** for embedding in documentation
- 🖥️ **ASCII preview** for rapid iteration in the terminal
- ✅ **Validation** with helpful error messages
- 🏗️ **Automatic layout** using a layer-based algorithm
- 🎯 **Node types** for different visual styles (service, database, external, queue)

## Installation

Build the project from source using Cargo:

```bash
cargo build --release
```

The compiled binary will be available at `target/release/diagrams`.

## Usage

### Compile DSL to SVG

Generate an SVG diagram from a DSL file:

```bash
diagrams compile input.dsl -o output.svg
```

**Example:**

Create a file `example.dsl`:
```
node "API Gateway" as api
node "Database" as db
api -> db : "SQL query"
```

Compile it:
```bash
diagrams compile example.dsl -o diagram.svg
```

### Preview in Terminal

View an ASCII art preview of your diagram:

```bash
diagrams preview input.dsl
```

This displays the diagram using Unicode box-drawing characters, perfect for quick iteration during development.

### Validate DSL

Check your DSL file for syntax and semantic errors:

```bash
diagrams validate input.dsl
```

The validator checks for:
- Valid syntax
- Undefined nodes in connections
- Duplicate node identifiers
- Self-referencing connections

## DSL Syntax Reference

### Node Declaration

Define a node with a display name and identifier:

```
node "<display name>" as <identifier>
```

**With type annotation:**

```
node "<display name>" as <identifier> [type: <node_type>]
```

Available node types:
- `service` - Service component (default)
- `database` - Database or data store
- `external` - External system or third-party service
- `queue` - Message queue or event bus

**Examples:**

```
node "API Gateway" as api
node "PostgreSQL" as db [type: database]
node "Redis Cache" as cache [type: database]
node "RabbitMQ" as queue [type: queue]
node "Stripe API" as stripe [type: external]
```

### Connection Declaration

Connect two nodes with an optional label:

```
<from_identifier> -> <to_identifier>
<from_identifier> -> <to_identifier> : "<label>"
```

**Examples:**

```
api -> db
api -> db : "SQL queries"
api -> cache : "GET/SET"
api -> queue : "publish events"
```

### Comments

Lines starting with `#` are treated as comments:

```
# This is a comment
node "API" as api
# Another comment
```

### Complete Example

```
# Define the architecture components
node "API Gateway" as api
node "User Service" as users [type: service]
node "PostgreSQL" as db [type: database]
node "Redis" as cache [type: database]
node "Message Queue" as mq [type: queue]
node "External Payment API" as payment [type: external]

# Define the connections
api -> users : "HTTP/REST"
users -> db : "SQL queries"
users -> cache : "session data"
users -> mq : "async events"
api -> payment : "process payment"
```

## Exit Codes

The tool uses specific exit codes for different error types:

- `0` - Success
- `1` - Syntax error (invalid DSL syntax)
- `2` - Semantic error (validation failure)
- `3` - I/O error (file not found, permission denied, etc.)

## Development

### Running Tests

```bash
cargo test --all
```

### Linting

```bash
cargo clippy -- -D warnings
```

### Formatting

```bash
cargo fmt
```

### Building Documentation

```bash
cargo doc --no-deps
```

## License

MIT

## Contributing

Contributions are welcome! Please ensure your code:
- Passes all tests (`cargo test`)
- Has no clippy warnings (`cargo clippy -- -D warnings`)
- Is properly formatted (`cargo fmt --check`)
