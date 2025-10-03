//! Template rendering for loco-rs code generation
//!
//! This module provides template rendering functionality for generating
//! model files, migrations, controllers, and views with integrated caching
//! for optimal performance.

use crate::error::BindingError;
use crate::field::FieldDefinition;
use crate::file_ops::read_file_to_string;
use std::collections::HashMap;
use once_cell::sync::Lazy;

pub use template_cache::{
    TemplateCache, ModelTemplateCache, ControllerTemplateCache,
    ViewTemplateCache, MigrationTemplateCache, get_template_cache,
    warm_template_cache
};

/// Render a model file template
pub fn render_model_template(model_name: &str, fields: &[FieldDefinition]) -> BindingResult<String> {
    let table_name = format!("{}s", model_name);

    let mut field_definitions = String::new();
    for field in fields {
        field_definitions.push_str(&render_field_definition(field));
        field_definitions.push('\n');
    }

    let template = format!(
        r#"use sea_orm::entity::prelude::*;
use serde::{{Deserialize, Serialize}};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "{}")]
pub struct Model {{
    #[sea_orm(primary_key)]
    pub id: i32,
{}}}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {{}}

impl ActiveModelBehavior for ActiveModel {{}}
"#,
        table_name,
        field_definitions
    );

    Ok(template)
}

/// Render a field definition for a model
fn render_field_definition(field: &FieldDefinition) -> String {
    let (rust_type, sea_orm_attrs) = match field.field_type {
        crate::field::FieldType::String => ("String", Some(r#"Some("String(Some(255))")"#)),
        crate::field::FieldType::I32 => ("i32", None),
        crate::field::FieldType::I64 => ("i64", None),
        crate::field::FieldType::F32 => ("f32", None),
        crate::field::FieldType::F64 => ("f64", None),
        crate::field::FieldType::Boolean => ("bool", None),
        crate::field::FieldType::DateTime => ("DateTime", Some(r#"Some("DateTime")"#)),
        crate::field::FieldType::Uuid => ("Uuid", Some(r#"Some("Uuid")"#)),
        crate::field::FieldType::Json => ("Json", Some(r#"Some("Json")"#)),
        crate::field::FieldType::Text => ("Text", Some(r#"Some("Text")"#)),
    };

    let mut field_line = format!("    #[sea_orm(");

    if let Some(attrs) = sea_orm_attrs {
        field_line.push_str("column_type = ");
        field_line.push_str(attrs);
    }

    // Add constraints
    for constraint in &field.constraints {
        match constraint {
            crate::field::FieldConstraint::Unique => {
                if field_line.len() > 11 { field_line.push_str(", "); }
                field_line.push_str("unique");
            }
            crate::field::FieldConstraint::PrimaryKey => {
                if field_line.len() > 11 { field_line.push_str(", "); }
                field_line.push_str("primary_key");
            }
            crate::field::FieldConstraint::Nullable => {
                if field_line.len() > 11 { field_line.push_str(", "); }
                field_line.push_str("nullable");
            }
            _ => {} // Skip other constraints for basic implementation
        }
    }

    field_line.push_str(")]\n");
    field_line.push_str(&format!("    pub {}: {},", field.name, rust_type));

    field_line
}

/// Render a migration file template
pub fn render_migration_template(model_name: &str, fields: &[FieldDefinition]) -> BindingResult<String> {
    let table_name = format!("{}s", model_name);

    let mut column_definitions = String::new();
    for field in fields {
        column_definitions.push_str(&render_migration_column(field));
        column_definitions.push('\n');
    }

    let template = format!(
        r#"use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {{
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {{
        manager
            .create_table(
                Table::create()
                    .table("{}::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new("{}::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
}}                    .to_owned(),
            )
            .await
    }}

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {{
        manager
            .drop_table(Table::drop().table("{}::Table).to_owned())
            .await
    }}
}}
"#,
        table_name,
        table_name,
        column_definitions,
        table_name
    );

    Ok(template)
}

/// Render a column definition for migration
fn render_migration_column(field: &FieldDefinition) -> String {
    let column_name = to_snake_case(&field.name);
    let column_type = match field.field_type {
        crate::field::FieldType::String => "string()",
        crate::field::FieldType::I32 => "integer()",
        crate::field::FieldType::I64 => "big_integer()",
        crate::field::FieldType::F32 => "float()",
        crate::field::FieldType::F64 => "double()",
        crate::field::FieldType::Boolean => "boolean()",
        crate::field::FieldType::DateTime => "timestamp_with_time_zone()",
        crate::field::FieldType::Uuid => "uuid()",
        crate::field::FieldType::Json => "json()",
        crate::field::FieldType::Text => "text()",
    };

    let mut column_def = format!(
        "                    .col(ColumnDef::new({}::{})\n                        .{}())",
        table_name,
        to_pascal_case(&field.name),
        column_type
    );

    // Add constraints
    for constraint in &field.constraints {
        match constraint {
            crate::field::FieldConstraint::Unique => {
                column_def.push_str("\n                        .unique()");
            }
            crate::field::FieldConstraint::Nullable => {
                column_def.push_str("\n                        .null()");
            }
            _ => {} // Skip other constraints for basic implementation
        }
    }

    if !field.is_nullable() && !field.is_primary_key() {
        column_def.push_str("\n                        .not_null()");
    }

    column_def
}

/// Render a controller template
pub fn render_controller_template(model_name: &str, _fields: &[FieldDefinition], include_views: bool) -> BindingResult<String> {
    let resource_name = to_plural(&model_name);
    let model_struct_name = to_pascal_case(&model_name);
    let controller_name = to_pascal_case(&resource_name);

    let template = if include_views {
        format!(
            r#"use loco_rs::prelude::*;
use crate::models::{};
use crate::views::{};

pub struct {} {{}}

#[async_trait::async_trait]
impl ControllerActions for {} {{
    async fn index(&self, req: &RequestContext) -> Result<Response> {{
        let items = {}::find().all(&req.db).await?;
        render!(Index, {{ "items": &items }})
    }}

    async fn show(&self, req: &RequestContext, id: i32) -> Result<Response> {{
        let item = {}::find_by_id(id).one(&req.db).await?;
        match item {{
            Some(item) => render!(Show, {{ "item": &item }}),
            None => render!(NotFound),
        }}
    }}

    async fn create(&self, req: &RequestContext) -> Result<Response> {{
        render!(Create)
    }}

    async fn store(&self, req: &RequestContext) -> Result<Response> {{
        redirect_to!(req, routes::{}::index())
    }}

    async fn edit(&self, req: &RequestContext, id: i32) -> Result<Response> {{
        let item = {}::find_by_id(id).one(&req.db).await?;
        match item {{
            Some(item) => render!(Edit, {{ "item": &item }}),
            None => render!(NotFound),
        }}
    }}

    async fn update(&self, req: &RequestContext, id: i32) -> Result<Response> {{
        redirect_to!(req, routes::{}::index())
    }}

    async fn delete(&self, req: &RequestContext, id: i32) -> Result<Response> {{
        redirect_to!(req, routes::{}::index())
    }}
}}
"#,
            model_struct_name, resource_name, controller_name,
            controller_name, model_struct_name, model_struct_name,
            resource_name, model_struct_name, resource_name, resource_name
        )
    } else {
        format!(
            r#"use loco_rs::prelude::*;
use crate::models::{};

pub struct {} {{}}

#[async_trait::async_trait]
impl ControllerActions for {} {{
    async fn index(&self, req: &RequestContext) -> Result<Response> {{
        let items = {}::find().all(&req.db).await?;
        format::json(serde_json::json!({{ "items": items }}))
    }}

    async fn show(&self, req: &RequestContext, id: i32) -> Result<Response> {{
        let item = {}::find_by_id(id).one(&req.db).await?;
        match item {{
            Some(item) => format::json(serde_json::json!(item)),
            None => format::json(serde_json::json!({{ "error": "Not found" }})),
        }}
    }}
}}
"#,
            model_struct_name, controller_name, controller_name,
            model_struct_name, model_struct_name
        )
    };

    Ok(template)
}

/// Render view templates
pub fn render_view_templates(model_name: &str, _fields: &[FieldDefinition]) -> BindingResult<HashMap<String, String>> {
    let mut templates = HashMap::new();

    let resource_name = to_plural(&model_name);
    let model_struct_name = to_pascal_case(&model_name);

    // List view
    templates.insert("list".to_string(), format!(
        r#"<h1>{}</h1>

<table class="table">
  <thead>
    <tr>
      <th>ID</th>
      <th>Name</th>
      <th>Actions</th>
    </tr>
  </thead>
  <tbody>
    {{% for item in items %}}
    <tr>
      <td>{{{{ item.id }}}}</td>
      <td>{{{{ item.name }}}}</td>
      <td>
        <a href="/{{{{ routes.{}.show(item.id) }}}}">Show</a>
        <a href="/{{{{ routes.{}.edit(item.id) }}}}">Edit</a>
      </td>
    </tr>
    {{% endfor %}}
  </tbody>
</table>

<a href="/{{{{ routes.{}.create() }}}">New {}</a>
"#,
        to_pascal_case(&resource_name),
        resource_name,
        resource_name,
        resource_name,
        model_struct_name
    ));

    // Show view
    templates.insert("show".to_string(), format!(
        r#"<h1>{}</h1>

<div>
  <p><strong>ID:</strong> {{{{ item.id }}}}</p>
  <p><strong>Name:</strong> {{{{ item.name }}}}</p>
</div>

<a href="/{{{{ routes.{}.index() }}}}">Back</a>
<a href="/{{{{ routes.{}.edit(item.id) }}}}">Edit</a>
"#,
        model_struct_name,
        resource_name,
        resource_name
    ));

    // Form view (for both create and edit)
    templates.insert("form".to_string(), format!(
        r#"<h1>{% if item %}Edit{}{% else %}New{}{% endif %}</h1>

<form method="post" action="{% if item %}/{{ routes.{}.update(item.id) }}{% else %}/{{ routes.{}.store() }}{% endif %}">
  <div class="form-group">
    <label for="name">Name:</label>
    <input type="text" id="name" name="name" value="{% if item %}{{ item.name }}{% endif %}" required>
  </div>

  <button type="submit" class="btn btn-primary">Save</button>
  <a href="/{{ routes.{}.index() }}">Cancel</a>
</form>
"#,
        model_struct_name,
        model_struct_name,
        resource_name,
        resource_name,
        resource_name
    ));

    Ok(templates)
}

// Helper functions for name conversion
fn to_snake_case(s: &str) -> String {
    s.to_lowercase()
}

fn to_pascal_case(s: &str) -> String {
    s.split('_')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
            }
        })
        .collect()
}

fn to_plural(s: &str) -> String {
    // Simple pluralization - in real implementation would be more sophisticated
    if s.ends_with('y') {
        format!("{}ies", &s[..s.len()-1])
    } else if s.ends_with('s') || s.ends_with("sh") || s.ends_with("ch") {
        format!("{}es", s)
    } else {
        format!("{}s", s)
    }
}