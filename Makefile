# Spring Batch RS - Makefile
# Provides convenient commands for building, testing, and maintaining the project

.PHONY: help build test check doc clean format lint audit bench examples install uninstall update-deps coverage website-install website-serve website-start website-build website-preview website-check website-clean website-deploy

# Default target
help:
	@echo "Spring Batch RS - Available Commands:"
	@echo ""
	@echo "Build & Development:"
	@echo "  build          - Build the project in release mode"
	@echo "  build-dev      - Build the project in debug mode"
	@echo "  clean          - Clean build artifacts"
	@echo "  install        - Install the crate globally"
	@echo "  uninstall      - Uninstall the crate"
	@echo ""
	@echo "Testing & Quality:"
	@echo "  test           - Run all tests with all features"
	@echo "  test-features  - Run tests with specific feature combinations"
	@echo "  check          - Run code quality checks (format, clippy, audit)"
	@echo "  format         - Format code with rustfmt"
	@echo "  lint           - Run clippy lints"
	@echo "  audit          - Run security audit"
	@echo "  coverage       - Generate test coverage report"
	@echo ""
	@echo "Documentation:"
	@echo "  doc            - Generate and open documentation"
	@echo "  doc-serve      - Serve documentation locally"
	@echo ""
	@echo "Examples:"
	@echo "  examples       - Build all examples"
	@echo "  examples-run   - Run specific examples"
	@echo ""
	@echo "Website (Astro + Starlight):"
	@echo "  website-install - Install website dependencies"
	@echo "  website-serve   - Start Astro dev server (http://localhost:4321)"
	@echo "  website-start   - Alias for website-serve"
	@echo "  website-build   - Build website for production"
	@echo "  website-preview - Preview production build"
	@echo "  website-check   - Type-check documentation"
	@echo "  website-clean   - Clean build artifacts"
	@echo "  website-deploy  - Deploy info (auto via GitHub Actions)"
	@echo ""
	@echo "Maintenance:"
	@echo "  update-deps    - Update dependencies"
	@echo "  bench          - Run benchmarks (if available)"
	@echo ""

# Build targets
build:
	cargo build --release --all-features

build-dev:
	cargo build --all-features

clean:
	cargo clean

# Testing targets
test:
	@echo "Running tests with all features..."
	cargo test --all-features

test-features:
	@echo "Running tests with specific features..."
	cargo test --features csv,json,xml
	cargo test --features rdbc-postgres,rdbc-mysql,rdbc-sqlite
	cargo test --features mongodb,orm
	cargo test --features zip,ftp

# Code quality targets
check:
	@echo "Running code quality checks..."
	@echo "Checking code format..."
	cargo fmt --all -- --check
	@echo "Running clippy lints..."
	cargo clippy --all-features -- -D warnings
	@echo "Running security audit..."
	cargo audit

format:
	@echo "Formatting code..."
	cargo fmt --all

lint:
	@echo "Running clippy lints..."
	cargo clippy --all-features -- -D warnings

audit:
	@echo "Running security audit..."
	cargo audit

# Documentation targets
doc:
	@echo "Generating documentation..."
	cargo clean
	cargo doc --no-deps --all-features --open

doc-serve:
	@echo "Serving documentation locally..."
	cargo doc --no-deps --all-features
	@echo "Documentation generated. Open target/doc/spring_batch_rs/index.html in your browser"

# Examples targets
examples:
	@echo "Building all examples..."
	cargo build --examples --all-features

examples-run:
	@echo "Available examples:"
	@echo "  make run-example-generate-csv-from-json"
	@echo "  make run-example-generate-json-from-csv"
	@echo "  make run-example-generate-json-from-xml"
	@echo "  make run-example-generate-xml-from-csv"
	@echo "  make run-example-mysql-writer"
	@echo "  make run-example-postgres-reader"
	@echo "  make run-example-mongodb-reader"
	@echo "  make run-example-orm-reader"
	@echo "  make run-example-ftp-transfer"
	@echo "  make run-example-zip-files"

run-example-generate-csv-from-json:
	cargo run --example generate_csv_file_from_json_file_with_processor --features csv,json

run-example-generate-json-from-csv:
	cargo run --example generate_json_file_from_csv_string_with_fault_tolerance --features csv,json

run-example-generate-json-from-xml:
	cargo run --example generate_json_file_from_xml_file --features json,xml

run-example-generate-xml-from-csv:
	cargo run --example generate_xml_from_csv_with_processor --features csv,xml

run-example-mysql-writer:
	cargo run --example mysql_writer_example --features rdbc-mysql

run-example-postgres-reader:
	cargo run --example log_records_from_postgres_database --features rdbc-postgres

run-example-mongodb-reader:
	cargo run --example read_records_from_mongodb_database --features mongodb

run-example-orm-reader:
	cargo run --example orm_reader_example --features orm

run-example-ftp-transfer:
	cargo run --example ftp_transfer_tasklet --features ftp

run-example-zip-files:
	cargo run --example zip_files_tasklet --features zip

# Installation targets
install:
	@echo "Installing crate globally..."
	cargo install --path . --all-features

uninstall:
	@echo "Uninstalling crate..."
	cargo uninstall spring-batch-rs

# Maintenance targets
update-deps:
	@echo "Updating dependencies..."
	cargo update

bench:
	@echo "Running benchmarks..."
	cargo bench --all-features

# Coverage target
coverage:
	@echo "Generating test coverage report..."
	@echo "Note: This requires cargo-tarpaulin to be installed"
	@echo "Install with: cargo install cargo-tarpaulin"
	cargo tarpaulin --all-features --out Html

# Website targets (Astro + Starlight)
website-install:
	@echo "Installing website dependencies..."
	cd website && npm install

website-serve:
	@echo "Starting Astro development server..."
	@echo "Visit http://localhost:4321/spring-batch-rs/"
	cd website && npm run dev

website-start:
	@echo "Starting Astro development server..."
	@echo "Visit http://localhost:4321/spring-batch-rs/"
	cd website && npm run start

website-build:
	@echo "Building Astro website for production..."
	cd website && npm run build

website-preview:
	@echo "Previewing production build..."
	cd website && npm run preview

website-check:
	@echo "Type-checking documentation..."
	cd website && npm run astro check

website-clean:
	@echo "Cleaning website build artifacts..."
	rm -rf website/dist website/.astro website/node_modules/.astro

website-deploy:
	@echo "Website deploys automatically via GitHub Actions when pushing to main"
	@echo "To deploy manually, push changes to the main branch"
	@echo "Website URL: https://spring-batch-rs.boussekeyt.dev/"

# CI/CD targets
ci: check test
	@echo "CI pipeline completed successfully"

# Development workflow
dev: format lint test
	@echo "Development workflow completed"

# Release preparation
release-prep: clean check test examples
	@echo "Release preparation completed"
	@echo "Ready to tag and publish"

# Quick development cycle
quick: format test
	@echo "Quick development cycle completed"

# Show project status
status:
	@echo "Project Status:"
	@echo "Rust version: $(shell rustc --version)"
	@echo "Cargo version: $(shell cargo --version)"
	@echo "Features enabled: $(shell cargo read-manifest | jq -r '.features | keys | join(", ")')"
	@echo "Dependencies: $(shell cargo tree --depth 1 | wc -l) direct dependencies"

# Additional useful targets
check-all: clean format lint test audit
	@echo "All checks completed successfully"

pre-commit: format lint test
	@echo "Pre-commit checks completed"

# Docker targets (if needed)
docker-build:
	@echo "Building Docker image..."
	docker build -t spring-batch-rs .

docker-run:
	@echo "Running Docker container..."
	docker run -it spring-batch-rs

# Backup and restore
backup:
	@echo "Creating backup of Cargo.lock..."
	cp Cargo.lock Cargo.lock.backup.$(shell date +%Y%m%d_%H%M%S)

restore:
	@echo "Restoring Cargo.lock from backup..."
	@ls -la Cargo.lock.backup.* | tail -1 | awk '{print $$9}' | xargs -I {} cp {} Cargo.lock

# Dependency analysis
deps-tree:
	@echo "Dependency tree:"
	cargo tree --depth 2

deps-outdated:
	@echo "Checking for outdated dependencies..."
	cargo outdated

# Performance analysis
profile:
	@echo "Profiling build performance..."
	cargo build --release --all-features --timings

# Clean various artifacts
clean-all: clean
	@echo "Cleaning additional artifacts..."
	rm -rf target/
	rm -rf coverage/
	rm -rf .cargo/
	@echo "All artifacts cleaned"

# Verify installation
verify:
	@echo "Verifying installation..."
	cargo --version
	rustc --version
	@echo "Checking if spring-batch-rs is available:"
	@cargo search spring-batch-rs 2>/dev/null || echo "Crate not found in registry"
