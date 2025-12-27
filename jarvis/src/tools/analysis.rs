use crate::tools::Tool;
use anyhow::Result;
use async_trait::async_trait;
use serde_json::{json, Value};
use std::fs;
use std::path::Path;

/// Tool to analyze code dependencies and imports
pub struct AnalyzeDependenciesTool;

#[async_trait]
impl Tool for AnalyzeDependenciesTool {
    fn name(&self) -> &str {
        "analyze_dependencies"
    }

    fn description(&self) -> &str {
        "Analyze dependencies and imports in a file or directory. Shows what external packages/modules are used."
    }

    async fn run(&self, input: Value) -> Result<Value> {
        let path_str = input["path"].as_str().ok_or_else(|| anyhow::anyhow!("Path is required"))?;
        let path = Path::new(path_str);
        
        let mut dependencies = Vec::new();
        
        if path.is_file() {
            dependencies.extend(analyze_file(path)?);
        } else if path.is_dir() {
            // Analyze all relevant files in the directory
            for entry in walkdir::WalkDir::new(path)
                .max_depth(3)
                .into_iter()
                .filter_map(|e| e.ok())
            {
                let entry_path = entry.path();
                if entry_path.is_file() {
                    if let Some(ext) = entry_path.extension() {
                        if ext == "rs" || ext == "js" || ext == "ts" || ext == "py" {
                            dependencies.extend(analyze_file(entry_path)?);
                        }
                    }
                }
            }
        }
        
        // Deduplicate
        dependencies.sort();
        dependencies.dedup();
        
        Ok(json!({
            "dependencies": dependencies,
            "count": dependencies.len()
        }))
    }
}

fn analyze_file(path: &Path) -> Result<Vec<String>> {
    let content = fs::read_to_string(path)?;
    let mut deps = Vec::new();
    
    let ext = path.extension().and_then(|s| s.to_str());
    
    match ext {
        Some("rs") => {
            // Rust: look for use statements
            for line in content.lines() {
                let trimmed = line.trim();
                if trimmed.starts_with("use ") {
                    if let Some(dep) = extract_rust_use(trimmed) {
                        deps.push(dep);
                    }
                }
            }
        }
        Some("js") | Some("ts") | Some("jsx") | Some("tsx") => {
            // JavaScript/TypeScript: look for import/require
            for line in content.lines() {
                let trimmed = line.trim();
                if trimmed.starts_with("import ") || trimmed.contains("require(") {
                    if let Some(dep) = extract_js_import(trimmed) {
                        deps.push(dep);
                    }
                }
            }
        }
        Some("py") => {
            // Python: look for import statements
            for line in content.lines() {
                let trimmed = line.trim();
                if trimmed.starts_with("import ") || trimmed.starts_with("from ") {
                    if let Some(dep) = extract_python_import(trimmed) {
                        deps.push(dep);
                    }
                }
            }
        }
        _ => {}
    }
    
    Ok(deps)
}

fn extract_rust_use(line: &str) -> Option<String> {
    // Extract the first part of use statement: "use crate::foo" -> "crate"
    let after_use = line.strip_prefix("use ")?.trim();
    let first_part = after_use.split("::").next()?;
    let cleaned = first_part.split('{').next()?.trim();
    if cleaned != "crate" && cleaned != "self" && cleaned != "super" {
        Some(cleaned.to_string())
    } else {
        None
    }
}

fn extract_js_import(line: &str) -> Option<String> {
    // Extract module name from import or require
    if line.contains("from") {
        // import X from 'module'
        if let Some(start) = line.rfind("from") {
            let after_from = &line[start + 4..].trim();
            return extract_quoted_string(after_from);
        }
    } else if line.contains("require(") {
        // require('module')
        if let Some(start) = line.find("require(") {
            let after_require = &line[start + 8..];
            return extract_quoted_string(after_require);
        }
    }
    None
}

fn extract_python_import(line: &str) -> Option<String> {
    if line.starts_with("from ") {
        // from module import X
        let after_from = line.strip_prefix("from ")?.trim();
        let module = after_from.split_whitespace().next()?;
        Some(module.to_string())
    } else if line.starts_with("import ") {
        // import module
        let after_import = line.strip_prefix("import ")?.trim();
        let module = after_import.split(&[' ', ','][..]).next()?;
        Some(module.to_string())
    } else {
        None
    }
}

fn extract_quoted_string(s: &str) -> Option<String> {
    let trimmed = s.trim();
    if let Some(start_quote) = trimmed.find(['\'', '"']) {
        let quote_char = trimmed.chars().nth(start_quote)?;
        let after_quote = &trimmed[start_quote + 1..];
        if let Some(end_quote) = after_quote.find(quote_char) {
            return Some(after_quote[..end_quote].to_string());
        }
    }
    None
}

/// Tool to find TODO, FIXME, and other code markers
pub struct FindCodeMarkersTool;

#[async_trait]
impl Tool for FindCodeMarkersTool {
    fn name(&self) -> &str {
        "find_code_markers"
    }

    fn description(&self) -> &str {
        "Find TODO, FIXME, HACK, NOTE, and XXX markers in code. Useful for identifying technical debt and pending work."
    }

    async fn run(&self, input: Value) -> Result<Value> {
        let path_str = input["path"].as_str().unwrap_or(".");
        let path = Path::new(path_str);
        
        let markers = vec!["TODO", "FIXME", "HACK", "NOTE", "XXX"];
        let mut results = Vec::new();
        
        for entry in walkdir::WalkDir::new(path)
            .max_depth(5)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let entry_path = entry.path();
            if entry_path.is_file() {
                if let Ok(content) = fs::read_to_string(entry_path) {
                    for (line_num, line) in content.lines().enumerate() {
                        for marker in &markers {
                            if line.contains(marker) {
                                results.push(json!({
                                    "file": entry_path.to_string_lossy(),
                                    "line": line_num + 1,
                                    "marker": marker,
                                    "content": line.trim()
                                }));
                            }
                        }
                    }
                }
            }
        }
        
        Ok(json!({
            "markers": results,
            "count": results.len()
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_extract_rust_use() {
        assert_eq!(extract_rust_use("use std::collections::HashMap;"), Some("std".to_string()));
        assert_eq!(extract_rust_use("use tokio::sync::Mutex;"), Some("tokio".to_string()));
        assert_eq!(extract_rust_use("use crate::foo::bar;"), None); // Internal
        assert_eq!(extract_rust_use("use super::something;"), None); // Internal
    }
    
    #[test]
    fn test_extract_js_import() {
        assert_eq!(extract_js_import("import React from 'react';"), Some("react".to_string()));
        assert_eq!(extract_js_import("const fs = require('fs');"), Some("fs".to_string()));
        assert_eq!(extract_js_import("import { useState } from 'react';"), Some("react".to_string()));
    }
    
    #[test]
    fn test_extract_python_import() {
        assert_eq!(extract_python_import("import numpy"), Some("numpy".to_string()));
        assert_eq!(extract_python_import("from django.core import management"), Some("django.core".to_string()));
        assert_eq!(extract_python_import("import os, sys"), Some("os".to_string()));
    }
}
