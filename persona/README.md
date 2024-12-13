# Persona

Server which stores, manages, and controls user data and actions.

## Development

In development, run `cargo sqlx db setup` (also make sure to have the sqlx cli installed,) to
avoid any SQLx errors in development. In runtime, this DB won't be used, however.
