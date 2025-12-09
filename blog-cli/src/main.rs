use blog_client::{BlogClientGrpc, BlogClientHttp, BlogClientTrait};
use clap::Parser;
use uuid::Uuid;

#[derive(Parser, Debug)]
struct Cli {
    #[clap(short, long)]
    grpc: bool,

    #[clap(short, long)]
    server: Option<String>,

    #[clap(subcommand)]
    command: Command,
}

#[derive(Parser, Debug)]
enum Command {
    Register {
        #[clap(long)]
        username: String,
        #[clap(long)]
        email: String,
        #[clap(long)]
        password: String,
    },
    Login {
        #[clap(long)]
        username: String,
        #[clap(long)]
        password: String,
    },
    ListPosts {
        #[clap(long)]
        limit: Option<u32>,
        #[clap(long)]
        offset: Option<u32>,
    },
    CreatePost {
        #[clap(long)]
        title: String,
        #[clap(long)]
        content: String,
    },
    GetPosts {
        #[clap(long)]
        author_id: Option<Uuid>,
        #[clap(long)]
        limit: Option<u32>,
        #[clap(long)]
        offset: Option<u32>,
    },
    UpdatePost {
        id: Uuid,
        #[clap(long)]
        title: Option<String>,
        #[clap(long)]
        content: Option<String>,
    },
    DeletePost {
        id: Uuid,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Cli::parse();

    // 1. Создаём клиента (один раз)
    let mut client: Box<dyn BlogClientTrait + Send + Sync> = if args.grpc {
        let endpoint = args.server.as_deref().unwrap_or("http://127.0.0.1:50051");
        Box::new(BlogClientGrpc::connect(endpoint).await?)
    } else {
        let endpoint = args.server.as_deref().unwrap_or("http://127.0.0.1:8080");
        Box::new(BlogClientHttp::connect(endpoint).await?)
    };

    // 2. Выполняем команду
    match args.command {
        Command::Register {
            username,
            email,
            password,
        } => {
            client.register(email, username, password).await?;
            println!("Successfully registered!");
        }
        Command::Login { username, password } => {
            client.login(username, password).await?;
            println!("Successfully logged in!");
        }
        Command::ListPosts { limit, offset } => {
            let posts = client.list_posts(None, limit, offset).await?;
            println!("Posts ({})", posts.len());
            for post in posts {
                println!("- [{}] {} (by {})", post.id, post.title, post.author_id);
            }
        }
        Command::CreatePost { title, content } => {
            let post = client.create_post(title, content).await?;
            println!("Post created! ID: {}", post.id);
        }
        Command::GetPosts {
            author_id,
            limit,
            offset,
        } => {
            let posts = client.list_posts(author_id, limit, offset).await?;
            for post in posts {
                println!("{}", post);
            }
        }
        Command::UpdatePost { id, title, content } => {
            let post = client.update_post(id, title, content).await?;
            println!("Post updated: {}", post)
        }
        Command::DeletePost { id } => {
            client.delete_post(id).await?;
            println!("Post deleted!")
        }
    }

    Ok(())
}
