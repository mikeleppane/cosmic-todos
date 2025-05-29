import logging

import azure.functions as func  # type: ignore

# Configure logging
logging.basicConfig(level=logging.INFO)

app = func.FunctionApp()

# This file serves as the entry point for the Azure Functions app
# Individual functions are defined in their respective directories
