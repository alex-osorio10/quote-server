# NBA Quote Server: A Pure Rust NBA Player Quote Webserver
Alex Osorio Trujillo, 05-2025

This project uses a Tokio/Axum/Askama/Sqlx/Sqlite stack to
serve NBA player quotes.

# Build and Run
Prerequisites
Rust

(Optional for Development) sqlx-cli: For managing database migrations and preparing queries. Install with this command:

cargo install sqlx-cli

# First-Time Setup and Run
Clone the Repository:

# git clone <your-repository-url>
# cd <your-repository-name>

Database URI:

By default, the application will create and use a SQLite database, located at `sqlite:db/quotes.db` by default (relative to the project root). You can override this by setting the `DATABASE_URL` environment variable or using the `--db-uri` command-line argument when running the application.

# Build, Initialize Database, and Run:

For the very first run when the `db/quotes.db` file does not yet exist, `sqlx`'s compile-time checks which verify your SQL queries might fail because it cannot find the database. To prevent this, you can run the command with the `SQLX_OFFLINE=true` environment variable. This tells sqlx to rely on its cached metadata in the `.sqlx` folder, which should be part of the repository for these checks, allowing the application to start, create the database, run migrations, and then initialize the data.

The command below will build the application, create the database if it doesn't exist, run migrations, load an initial collection of quotes from `assets/static/quotes.json`, and start the server.

# For Windows PowerShell:
    ```powershell
    $env:SQLX_OFFLINE="true"; cargo run -- --init-from assets/static/quotes.json; Remove-Item Env:\SQLX_OFFLINE
    ```
# For Linux/macOS or other shells (like Git Bash on Windows):
    ```bash
    SQLX_OFFLINE=true cargo run -- --init-from assets/static/quotes.json
    ```

You can also release an optimized build that takes a bit longer to compile with this command:

# For Windows PowerShell:
    ```powershell
    $env:SQLX_OFFLINE="true"; cargo run --release -- --init-from assets/static/quotes.json; Remove-Item Env:\SQLX_OFFLINE
    ```
# For Linux/macOS or other shells (like Git Bash on Windows):
    ```bash
    SQLX_OFFLINE=true cargo run --release -- --init-from assets/static/quotes.json
    ```

After running, the server should be accessible, typically at `http://127.0.0.1:3000`.

# Further Runs
Once the database file (`db/quotes.db`) has been created by the first run, you can run the server without the `SQLX_OFFLINE=true` variable and without the `--init-from` flag (unless you want to re-initialize with potentially updated JSON data):

cargo run

Or for a release build:

cargo run --release

# Clearing and Re-initializing

If you change up `assets/static/quotes.json` and want to reload all quotes into a fresh database you must:

1.  Stop the server if it's running.
2.  Delete the existing database files (e.g., `db/quotes.db`, `db/quotes.db-shm`, `db/quotes.db-wal`).
3.  Run the initialization command again (using `SQLX_OFFLINE=true` as it's like a "first run" for the database file):
# For Windows PowerShell:
        ```powershell
        $env:SQLX_OFFLINE="true"; cargo run -- --init-from assets/static/quotes.json; Remove-Item Env:\SQLX_OFFLINE
        ```
# For Linux/macOS or other shells (like Git Bash on Windows):
        ```bash
        SQLX_OFFLINE=true cargo run -- --init-from assets/static/quotes.json
        ```

If you encounter build issues, `cargo clean` can be used to remove old build artifacts before rebuilding.

# Clearing and Re-initializing

If you need to make a new database (for example, if you've changed `assets/static/quotes.json` and want to reload all quotes into a completely fresh database, or to reset to a clean state), you should first delete the old database files and then restart the application with the initialization command.

1.  Stop the server if it's running.
2.  Delete the existing database files from your project's root directory, use the appropriate command for your system:
# For Windows PowerShell:
        ```powershell
        Remove-Item -Path db/quotes.db, db/quotes.db-shm, db/quotes.db-wal -ErrorAction SilentlyContinue
        ```
        (The `-ErrorAction SilentlyContinue` will prevent errors if some of the files don't exist).
# For Linux/macOS or other shells (like Git Bash on Windows):
        ```bash
        rm -f db/quotes.db db/quotes.db-shm db/quotes.db-wal
        ```
3.  Restart the application with the initialization command (using `SQLX_OFFLINE=true` as it's like a "first run" for the database file):
# For Windows PowerShell:
        ```powershell
        $env:SQLX_OFFLINE="true"; cargo run -- --init-from assets/static/quotes.json; Remove-Item Env:\SQLX_OFFLINE
        ```
# For Linux/macOS or other shells (like Git Bash on Windows):
        ```bash
        SQLX_OFFLINE=true cargo run -- --init-from assets/static/quotes.json
        ```
If you encounter build issues after other changes, `cargo clean` can be used to remove old build artifacts before rebuilding.

## Development

SQLX Migrations: This project uses `sqlx` migrations. To add a new migration named `<name>`:

    sqlx migrate add -r <name>

Then edit the made migration files in the migrations directory.

SQLX Query Checking: `sqlx` checks queries against the database schema at compile time. If you modify 
the schemas (after running migrations) or your SQL queries, run:

    cargo sqlx prepare

You might need to specify the database URL if it's not found automatically by `sqlx-cli` (`cargo sqlx prepare --database-url sqlite:db/quotes.db` or by ensuring `DATABASE_URL` is in your `.env` file). This updates the `.sqlx` directory.

Committing Changes: You may need to add changes in the `.sqlx` directory and the `migrations` directory to your git commits with this command:

    git add .sqlx migrations

## License
This work is made available under the "Apache 2.0 or MIT License". See the file `LICENSE.txt` in this distribution for license terms.