DATABASE_URL := "sqlite:database.db"

# Run the application
run:
  DATABASE_URL={{DATABASE_URL}} cargo run
