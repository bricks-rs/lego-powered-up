use rhai::{AST,Engine};
use bevy::prelude::*;

#[derive(Default)]
pub struct InputMappingScript {
	script: String,
	script_ast: Option<AST>,
}

pub fn run_scripts(mut script: ResMut<InputMappingScript>) {
	let engine = Engine::new();
	if script.script_ast.is_none() {
		// Script has changed; recompile
		info!("Script changed, recompiling...");
		match engine.compile(&script.script) {
			Ok(ast) => script.script_ast = Some(ast),
			Err(e) => error!("Error compiling script: {}", e),
		}
	}

	if let Some(ast) = &script.script_ast {
		// Only enter here if the script has been successfully, either
		// on this run or a previous one

		match engine.eval_ast::<u8>(&ast) {
			Ok(_) => {},
			Err(e) => error!("Error running script: {}", e),
		}
	}
}
