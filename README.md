NBA Quote Server: A Pure Rust NBA Player Quote Webserver
Alex Osorio Trujillo, June 2025
This project uses a Tokio/Axum/Askama/Sqlx/Sqlite stack to serve NBA player quotes.

Prerequisites
Rust: Ensure you have a recent version of the Rust toolchain installed.

(Optional) sqlx-cli: For managing database migrations. Install with:

cargo install sqlx-cli.

1. Clone the Repository
git clone <your-repository-url>
cd <your-repository-name>

2. Create Secret Files (Required for First Run)
The application requires two secret files for JWT authentication and quote registration, which are not checked into version control for security. You must create them manually.

A. Create the secrets directory:
In the root of the project, create a new folder named secrets.

B. Create the secret files:
Inside the secrets directory, create two new files:

jwt_secret.txt: This file will contain the secret key for signing JSON Web Tokens.

reg_password.txt: This file will contain the password required to use the /register endpoint to get a JWT.

C. Add content to the files:

Open jwt_secret.txt and add a long, random, and secret string.

Open reg_password.txt and add the password you want to use for registration.

3. First-Time Run
This command will build the application, create the database, run migrations, and load the initial quotes from assets/static/quotes.json.

We use SQLX_OFFLINE=true because sqlx verifies queries at compile time, which would fail if the database doesn't exist yet. It tells it to trust the cached query information in the .sqlx directory.

For Windows PowerShell:

$env:SQLX_OFFLINE="true"; cargo run -- --init-from assets/static/quotes.json; Remove-Item Env:\SQLX_OFFLINE

For Linux/macOS or Git Bash on Windows:

SQLX_OFFLINE=true cargo run -- --init-from assets/static/quotes.json

After running, the server should be accessible at http://127.0.0.1:3000.

4. Subsequent Runs
Once the database and secrets are set up, you can start the server with this command:

cargo run

For an optimized release build:

cargo run --release

Database and Project Management
Clearing and Re-initializing
If you want to reset the database to a clean state (for example, after updating quotes.json), follow these steps:

Stop the server if it's running.

Delete the database files. These are located in the db/ directory.

For Windows PowerShell:

Remove-Item -Path "db" -Recurse -Force -ErrorAction SilentlyContinue

For Linux/macOS or Git Bash:

rm -rf db

Re-run the first-time setup command to rebuild and re-initialize the database.

Cleaning Build Artifacts
The /target directory, which contains all compiled code, is also ignored by Git. If you encounter strange build issues, you can clear it with the standard Cargo command:

cargo clean

Development with sqlx-cli
Add a new migration:

sqlx migrate add -r <migration_name>

Update query information after schema changes:

cargo sqlx prepare


This ensures your SQL queries are still valid against the new database schema. 

Docker Deployment
1. Build the Docker Image
First, ensure Docker is installed and running on your system. Then, build the image from the project root:

docker build -t quote-server .

2. Run the Docker Container
The application inside the container requires the JWT and registration secrets to be passed as environment variables.

For Linux/macOS or Git Bash:

docker run -p 3000:3000 \
  -e JWT_SECRETFILE="$(cat secrets/jwt_secret.txt)" \
  -e REG_PASSWORD="$(cat secrets/reg_password.txt)" \
  --name my-quote-server \
  quote-server

For Windows PowerShell:

docker run -p 3000:3000 `
  -e "JWT_SECRETFILE=$(Get-Content secrets\jwt_secret.txt)" `
  -e "REG_PASSWORD=$(Get-Content secrets\reg_password.txt)" `
  --name my-quote-server `
  quote-server

This method is secure because your secrets are injected at runtime and are not stored in the Docker image itself.

License
This work is made available under the "Apache 2.0 or MIT License". See the file LICENSE.txt in this distribution for license terms.