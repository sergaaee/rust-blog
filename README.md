# rust-blog

```bash
cargo build
```

## 1) Server:
1. Create db via psql (use blog-api as an example)
2. Fill the `````.env````` file from ```.env.example```
```bash
cd blog-server && cargo run
```

## 2) CLI
1. Login
```bash
cargo run -- login --username proverka --password proverka
```
2. Register
```bash
cargo run -- register --username proverka --email proverka@duck.com --password proverka
```
3. Create post
```bash 
cargo run -- create-post --title <String> --content <String>
```
4. Get post
```bash
cargo run -- get-post --id <UUID>
```
5. Update post
```bash
cargo run -- update-post --id <UUID> --title <String> --content <String>
```
6. Delete post
```bash
cargo run -- delete-post --id <UUID>
```
7. List posts
```bash
cargo run -- list-posts
```

## 3) Frontend
1. install dependencies
```bash
rustup toolchain install stable
rustup target add wasm32-unknown-unknown
curl -sSL http://dioxus.dev/install.sh | sh
```
2. Run the app. You can change port accordingly to allowed CORS in .env
```bash
cd blog-wasm && dx serve --port 52734
```