# <p align="center"> 🌌 Cosmic Todos </p>

<div align="center">

![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=for-the-badge&logo=rust&logoColor=white)
![WebAssembly](https://img.shields.io/badge/WebAssembly-654FF0?style=for-the-badge&logo=WebAssembly&logoColor=white)
![Docker](https://img.shields.io/badge/docker-%230db7ed.svg?style=for-the-badge&logo=docker&logoColor=white)
![Azure](https://img.shields.io/badge/azure-%230072C6.svg?style=for-the-badge&logo=microsoftazure&logoColor=white)

## A modern, beautiful family task management application built with Rust, Leptos and Azure Cosmos DB*

![Cosmic Todos](images/cosmic-rust-logo.png)


</div>

---

## 📖 About

Cosmic Todos is a sleek, modern web application designed for family task management. Built with cutting-edge Rust web technologies, it combines the performance of WebAssembly with the elegance of reactive UI components to create a delightful user experience.

### 🎯 Why Cosmic Todos?

- **Family-First Design**: Built specifically for family collaboration with multi-user support
- **Modern Architecture**: Leverages Rust's type safety and performance in the browser
- **Beautiful Interface**: Stunning gradient designs and smooth animations
- **Developer Experience**: Type-safe development with excellent tooling

## ✨ Features

### 🎨 **Beautiful User Interface**

- Modern gradient-based design system
- Responsive layout that works on all devices
- Smooth animations and micro-interactions
- Dark mode support (coming soon)

### 📋 **Comprehensive Task Management**

- Create, edit, and delete tasks with rich metadata
- Set due dates and times with timezone awareness
- Track task status (Not Started, In Progress, Completed, Blocked)
- Visual indicators for overdue tasks

### 👥 **Family-Focused**

- Multi-user support for family members
- Assign tasks to specific family members
- Filter tasks by assignee
- Shared family dashboard

### 🔐 **Secure Authentication**

- Environment-based configuration
- Session management
- Secure login flow

### 📱 **Responsive & Accessible**

- Mobile-first responsive design
- Keyboard navigation support
- Screen reader friendly
- Touch-friendly interface


## 📸 Screenshots

<div align="center">

![Cosmic Todos App Screenshot](images/app-screenshot.png)

*Cosmic Todos in action*

</div>


## 🏗️ Architecture

Cosmic Todos follows a modern full-stack architecture leveraging Rust's ecosystem for both frontend and backend components, with Azure cloud services providing scalable infrastructure.

![Architecture Diagram](images/cosmis-rust-arch.png)

### Architecture Overview

The application is built on a **three-tier architecture** with clear separation of concerns:

- **Frontend Layer**: Leptos-based reactive UI compiled to WebAssembly, providing type-safe client-side interactions with excellent performance
- **API Layer**: Rust-based serverless functions hosted on Azure Functions, handling business logic and data validation
- **Data Layer**: Azure Cosmos DB providing globally distributed, multi-model database capabilities with automatic scaling

**Key Architectural Benefits:**
- **Type Safety**: End-to-end Rust ensures compile-time error detection and prevents runtime errors
- **Performance**: WebAssembly frontend delivers near-native performance in the browser
- **Scalability**: Serverless architecture automatically scales based on demand
- **Global Distribution**: Azure Cosmos DB enables low-latency access worldwide
- **Developer Experience**: Shared types and models between frontend and backend reduce duplication and increase maintainability

The architecture supports modern development practices including continuous deployment, containerization, and cloud-native patterns while maintaining the performance and safety guarantees that Rust provides.

## 🛠️ Technology Stack

| Category | Technology |
|----------|------------|
| **Frontend** | [Leptos](https://leptos.dev/) - Reactive Rust web framework |
| **Language** | [Rust](https://www.rust-lang.org/) with WebAssembly |
| **Styling** | [Tailwind CSS](https://tailwindcss.com/) |
| **Database** | [Azure Cosmos DB](https://azure.microsoft.com/en-us/services/cosmos-db/) |
| **Hosting** | [Azure Static Web Apps](https://azure.microsoft.com/en-us/services/app-service/static/) |
| **Build Tool** | [Cargo Leptos](https://github.com/leptos-rs/cargo-leptos) |
| **Date/Time** | [Chrono](https://docs.rs/chrono/) |
| **Routing** | [Leptos Router](https://docs.rs/leptos_router/) |
| **Task Runner** | [Just](https://github.com/casey/just) |
| **Container** | [Docker](https://www.docker.com/) |

## 🚀 Getting Started

### Prerequisites

- [Rust](https://rustup.rs/) (latest stable)
- [Node.js](https://nodejs.org/) (for Tailwind CSS)
- [Just](https://github.com/casey/just) (optional, for task automation)
- [Azure CLI](https://docs.microsoft.com/en-us/cli/azure/) (for Azure deployment)

### 🔧 Installation

1. **Clone the repository**

   ```bash
   git clone https://github.com/yourusername/cosmic-todos.git
   cd cosmic-todos
    ```

2. **Install Rust dependencies**

  ```bash
  cargo install cargo-leptos
  rustup target add wasm32-unknown-unknown
  ```

3. **Set up your environment variables**

  ```bash
  cp .env.example .env
  # Edit .env with your credentials
  export COSMIC_USERNAME=your_username
  export COSMIC_PASSWORD=your_password
  export AZURE_COSMOS_CONNECTION_STRING=your_cosmos_connection_string
  ```

4. **Run the development server**

```bash
just run
# or 
cargo leptos watch
```

5. **Open your browser and navigate to `http://localhost:3000`**

☁️ Azure Deployment
Deploy to Azure Static Web Apps:

```bash
just deploy
# or
# Login to Azure
az login

# Create resource group
az group create --name cosmic-todos-rg --location eastus

# Deploy using Azure CLI
az staticwebapp create \
  --name cosmic-todos \
  --resource-group cosmic-todos-rg \
  --source https://github.com/yourusername/cosmic-todos \
  --location eastus \
  --branch main \
  --app-location "/" \
  --api-location "api" \
  --output-location "dist"
```

🐳 Docker Setup

```bash
# Build the image
docker build -t cosmic-todos .

# Run the container
docker run -p 3000:80 \
  -e COSMIC_USERNAME=admin \
  -e COSMIC_PASSWORD=password \
  -e AZURE_COSMOS_CONNECTION_STRING=your_connection_string \
  cosmic-todos
```

📋 Development
Available Commands
This project uses Just for task automation:

```bash
just --list                 # Show all available commands
just format                 # Format code with rustfmt
just lint                   # Lint code with clippy
just check                  # Run format and lint checks
just run                    # Start development server
just build                  # Build in debug mode
just build-release          # Build in release mode
just test                   # Run tests
just clean                  # Clean build artifacts
just setup                  # Set up development environment
just production             # Full production build
just deploy                 # Deploy to Azure
just docker-build           # Build Docker image
```

If you don't have Just installed, check the [Just manual](https://just.systems/man/en/introduction.html)

🎨 Styling
The project uses Tailwind CSS with custom configurations:

Color Palette: Purple, fuchsia, and indigo gradients
Components: Reusable UI components with consistent styling
Responsive: Mobile-first approach with breakpoint utilities

🧪 Testing

```bash
# Run all tests
just test

# Run specific test
cargo test test_name

# Run with coverage
cargo tarpaulin --out html
```

📁 Project Structure

```bash
cosmic-todos/
├── src/
│   ├── app.rs              # Main application component
│   ├── components/         # Reusable UI components
│   ├── pages/             # Page components
│   └── lib.rs             # Library root
├── style/                 # Tailwind CSS styles
├── public/                # Static assets
├── api/                   # Azure Functions API
├── Cargo.toml             # Rust dependencies
├── Dockerfile             # Container configuration
├── justfile               # Task automation
├── staticwebapp.config.json # Azure SWA configuration
└── README.md              # This file
```

🔧 Configuration
Environment Variables

| Variable | Description | Required |
|----------|-------------|----------|
| COSMIC_USERNAME    | Application username | Yes      |
| COSMIC_PASSWORD    | Application password | Yes      |
| AZURE_COSMOS_CONNECTION_STRING | Cosmos DB connection string | Yes      |
| RUST_LOG | Log level (info, debug, warn, error) | No       |  

![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)

🙏 Acknowledgments

- Leptos - For the amazing Rust web framework  
- Tailwind CSS - For the utility-first CSS framework
- Azure - For reliable cloud infrastructure
- Rust Community - For the incredible ecosystem
