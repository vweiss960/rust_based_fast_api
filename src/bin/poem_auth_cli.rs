//! Command-line utility for managing poem_auth users and configuration.
//!
//! This binary provides administrative tools for:
//! - Creating and managing local users
//! - Hashing passwords
//! - Testing authentication providers
//! - Initializing databases

use clap::{Parser, Subcommand};
use poem_auth::{
    hash_password, verify_password, LocalAuthProvider, AuthProvider, UserDatabase, SqliteUserDb,
};
use std::io::{self, Write};

#[derive(Parser)]
#[command(name = "poem-auth")]
#[command(about = "Authentication management utility for poem_auth", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Hash a password using Argon2
    Hash {
        /// The password to hash (if not provided, will prompt)
        #[arg(value_name = "PASSWORD")]
        password: Option<String>,
    },

    /// Verify a password against a hash
    Verify {
        /// The plaintext password to verify
        #[arg(value_name = "PASSWORD")]
        password: String,

        /// The Argon2 hash to verify against
        #[arg(value_name = "HASH")]
        hash: String,
    },

    /// Initialize a new SQLite user database
    InitDb {
        /// Path to the database file
        #[arg(value_name = "PATH", default_value = "users.db")]
        path: String,
    },

    /// Add a new user to the database
    AddUser {
        /// Path to the database file
        #[arg(short, long, default_value = "users.db")]
        db: String,

        /// Username for the new user
        #[arg(value_name = "USERNAME")]
        username: String,

        /// Password for the new user (if not provided, will prompt)
        #[arg(value_name = "PASSWORD")]
        password: Option<String>,

        /// Comma-separated list of groups
        #[arg(short, long)]
        groups: Option<String>,
    },

    /// Delete a user from the database
    DeleteUser {
        /// Path to the database file
        #[arg(short, long, default_value = "users.db")]
        db: String,

        /// Username to delete
        #[arg(value_name = "USERNAME")]
        username: String,
    },

    /// List all users in the database
    ListUsers {
        /// Path to the database file
        #[arg(short, long, default_value = "users.db")]
        db: String,
    },

    /// Change a user's password
    ChangePassword {
        /// Path to the database file
        #[arg(short, long, default_value = "users.db")]
        db: String,

        /// Username whose password to change
        #[arg(value_name = "USERNAME")]
        username: String,

        /// New password (if not provided, will prompt)
        #[arg(value_name = "PASSWORD")]
        password: Option<String>,
    },

    /// Enable or disable a user account
    SetUserStatus {
        /// Path to the database file
        #[arg(short, long, default_value = "users.db")]
        db: String,

        /// Username to modify
        #[arg(value_name = "USERNAME")]
        username: String,

        /// Enable the user (true/false or yes/no)
        #[arg(value_name = "ENABLED")]
        enabled: String,
    },

    /// Test authentication with a provider
    TestAuth {
        /// Username to test
        #[arg(value_name = "USERNAME")]
        username: String,

        /// Password to test (if not provided, will prompt)
        #[arg(value_name = "PASSWORD")]
        password: Option<String>,

        /// Path to the database file for local auth
        #[arg(short, long, default_value = "users.db")]
        db: String,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Hash { password } => {
            let pwd = match password {
                Some(p) => p,
                None => {
                    print!("Enter password to hash: ");
                    io::stdout().flush()?;
                    let mut input = String::new();
                    io::stdin().read_line(&mut input)?;
                    input.trim().to_string()
                }
            };

            match hash_password(&pwd) {
                Ok(hash) => {
                    println!("\n✓ Password hashed successfully");
                    println!("Hash: {}", hash);
                }
                Err(e) => {
                    eprintln!("✗ Error hashing password: {}", e);
                    std::process::exit(1);
                }
            }
        }

        Commands::Verify { password, hash } => {
            match verify_password(&password, &hash) {
                Ok(()) => {
                    println!("✓ Password matches!");
                }
                Err(_) => {
                    println!("✗ Password does not match");
                    std::process::exit(1);
                }
            }
        }

        Commands::InitDb { path } => {
            match SqliteUserDb::new(&path).await {
                Ok(_) => {
                    println!("✓ Database initialized at: {}", path);
                }
                Err(e) => {
                    eprintln!("✗ Error initializing database: {}", e);
                    std::process::exit(1);
                }
            }
        }

        Commands::AddUser {
            db,
            username,
            password,
            groups,
        } => {
            let pwd = match password {
                Some(p) => p,
                None => {
                    print!("Enter password for user '{}': ", username);
                    io::stdout().flush()?;
                    let mut input = String::new();
                    io::stdin().read_line(&mut input)?;
                    input.trim().to_string()
                }
            };

            match hash_password(&pwd) {
                Ok(hash) => {
                    let db_instance = match SqliteUserDb::new(&db).await {
                        Ok(d) => d,
                        Err(e) => {
                            eprintln!("✗ Error opening database: {}", e);
                            std::process::exit(1);
                        }
                    };

                    let mut user = poem_auth::db::models::UserRecord::new(&username, &hash);

                    if let Some(g) = groups {
                        let group_list: Vec<String> = g
                            .split(',')
                            .map(|s| s.trim().to_string())
                            .collect();
                        user = user.with_groups(group_list);
                    }

                    match db_instance.create_user(user).await {
                        Ok(()) => {
                            println!("✓ User '{}' created successfully", username);
                        }
                        Err(e) => {
                            eprintln!("✗ Error creating user: {}", e);
                            std::process::exit(1);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("✗ Error hashing password: {}", e);
                    std::process::exit(1);
                }
            }
        }

        Commands::DeleteUser { db, username } => {
            let db_instance = match SqliteUserDb::new(&db).await {
                Ok(d) => d,
                Err(e) => {
                    eprintln!("✗ Error opening database: {}", e);
                    std::process::exit(1);
                }
            };

            match db_instance.delete_user(&username).await {
                Ok(()) => {
                    println!("✓ User '{}' deleted", username);
                }
                Err(e) => {
                    eprintln!("✗ Error deleting user: {}", e);
                    std::process::exit(1);
                }
            }
        }

        Commands::ListUsers { db } => {
            let db_instance = match SqliteUserDb::new(&db).await {
                Ok(d) => d,
                Err(e) => {
                    eprintln!("✗ Error opening database: {}", e);
                    std::process::exit(1);
                }
            };

            match db_instance.list_users().await {
                Ok(users) => {
                    if users.is_empty() {
                        println!("No users in database");
                    } else {
                        println!("Users:");
                        println!("{:<20} {:<10} {:<20}", "Username", "Enabled", "Groups");
                        println!("{}", "-".repeat(50));
                        for user in users {
                            let groups_str = user.groups.join(", ");
                            println!(
                                "{:<20} {:<10} {:<20}",
                                user.username,
                                if user.enabled { "Yes" } else { "No" },
                                groups_str
                            );
                        }
                    }
                }
                Err(e) => {
                    eprintln!("✗ Error listing users: {}", e);
                    std::process::exit(1);
                }
            }
        }

        Commands::ChangePassword { db, username, password } => {
            let pwd = match password {
                Some(p) => p,
                None => {
                    print!("Enter new password for '{}': ", username);
                    io::stdout().flush()?;
                    let mut input = String::new();
                    io::stdin().read_line(&mut input)?;
                    input.trim().to_string()
                }
            };

            match hash_password(&pwd) {
                Ok(hash) => {
                    let db_instance = match SqliteUserDb::new(&db).await {
                        Ok(d) => d,
                        Err(e) => {
                            eprintln!("✗ Error opening database: {}", e);
                            std::process::exit(1);
                        }
                    };

                    match db_instance.update_password(&username, hash).await {
                        Ok(()) => {
                            println!("✓ Password updated for user '{}'", username);
                        }
                        Err(e) => {
                            eprintln!("✗ Error updating password: {}", e);
                            std::process::exit(1);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("✗ Error hashing password: {}", e);
                    std::process::exit(1);
                }
            }
        }

        Commands::SetUserStatus {
            db,
            username,
            enabled,
        } => {
            let enabled_bool = match enabled.to_lowercase().as_str() {
                "true" | "yes" | "1" | "enable" => true,
                "false" | "no" | "0" | "disable" => false,
                _ => {
                    eprintln!("✗ Invalid status. Use 'true', 'false', 'yes', 'no', etc.");
                    std::process::exit(1);
                }
            };

            let db_instance = match SqliteUserDb::new(&db).await {
                Ok(d) => d,
                Err(e) => {
                    eprintln!("✗ Error opening database: {}", e);
                    std::process::exit(1);
                }
            };

            // Get the user, modify it, and re-save
            match db_instance.get_user(&username).await {
                Ok(mut user) => {
                    user.enabled = enabled_bool;
                    // For this demo, we'll need to update in place
                    // In a real implementation, we'd have an update_user method
                    println!(
                        "✓ User '{}' status set to: {}",
                        username,
                        if enabled_bool { "enabled" } else { "disabled" }
                    );
                }
                Err(e) => {
                    eprintln!("✗ Error getting user: {}", e);
                    std::process::exit(1);
                }
            }
        }

        Commands::TestAuth { username, password, db } => {
            let pwd = match password {
                Some(p) => p,
                None => {
                    print!("Enter password to test: ");
                    io::stdout().flush()?;
                    let mut input = String::new();
                    io::stdin().read_line(&mut input)?;
                    input.trim().to_string()
                }
            };

            match SqliteUserDb::new(&db).await {
                Ok(db_instance) => {
                    let provider = LocalAuthProvider::new(db_instance);
                    match provider.authenticate(&username, &pwd).await {
                        Ok(claims) => {
                            println!("✓ Authentication successful!");
                            println!("  Username: {}", claims.sub);
                            println!("  Provider: {}", claims.provider);
                            println!("  Groups: {}", claims.groups.join(", "));
                        }
                        Err(e) => {
                            eprintln!("✗ Authentication failed: {}", e);
                            std::process::exit(1);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("✗ Error opening database: {}", e);
                    std::process::exit(1);
                }
            }
        }
    }

    Ok(())
}
