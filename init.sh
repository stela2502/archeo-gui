#!/usr/bin/env bash

set -euo pipefail

mkdir -p src/gui
mkdir -p src/backend/models
mkdir -p src/backend/data_handlers
mkdir -p src/backend/mysql

touch \
    src/main.rs \
    \
    src/gui/mod.rs \
    src/gui/app.rs \
    src/gui/wizard.rs \
    src/gui/review.rs \
    src/gui/file_detail.rs \
    src/gui/chat_panel.rs \
    \
    src/backend/mod.rs \
    src/backend/archeo_backend.rs \
    src/backend/case_service.rs \
    src/backend/analysis_runner.rs \
    src/backend/prompt_builder.rs \
    src/backend/ai_client.rs \
    \
    src/backend/models/mod.rs \
    src/backend/models/project.rs \
    src/backend/models/file_record.rs \
    src/backend/models/folder_record.rs \
    src/backend/models/annotation.rs \
    src/backend/models/interaction.rs \
    src/backend/models/profile.rs \
    src/backend/models/analysis_result.rs \
    \
    src/backend/data_handlers/mod.rs \
    src/backend/data_handlers/handler_registry.rs \
    src/backend/data_handlers/generic_file_handler.rs \
    src/backend/data_handlers/text_handler.rs \
    src/backend/data_handlers/table_handler.rs \
    src/backend/data_handlers/code_handler.rs \
    src/backend/data_handlers/bioinformatics_handler.rs \
    \
    src/backend/mysql/mod.rs \
    src/backend/mysql/connection.rs \
    src/backend/mysql/project_repository.rs \
    src/backend/mysql/file_repository.rs \
    src/backend/mysql/annotation_repository.rs \
    src/backend/mysql/interaction_repository.rs

echo "Archeo GUI project structure created."
