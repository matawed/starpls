use crate::{Database, FilePosition, Location};
use starpls_common::{parse, Db, FileId};
use starpls_hir::{lower, Declaration, Name, Resolver};
use starpls_syntax::ast::{self, AstNode, AstPtr};

pub(crate) fn goto_definition(
    db: &Database,
    FilePosition { file_id, pos }: FilePosition,
) -> Option<Vec<Location>> {
    let file = db.get_file(file_id)?;
    let res = parse(db, file);
    let parent = res
        .inner(db)
        .syntax()
        .token_at_offset(pos)
        .next()?
        .parent()?;

    // For now, we only handle identifiers.
    if let Some(name) = ast::Name::cast(parent) {
        let ptr = AstPtr::new(&ast::Expression::cast(name.syntax().clone())?);
        let info = lower(db, res);
        let source_map = info.source_map(db);
        let expr = source_map.expr_map.get(&ptr).cloned()?;
        let name = Name::from_ast_node(db, name);
        let resolver = Resolver::new_for_expr(db, info, expr);
        return Some(
            resolver
                .resolve_name(name)?
                .into_iter()
                .flat_map(|decl| match decl {
                    Declaration::Function { id } => {
                        source_map.stmt_map_back.get(&id).map(|ptr| Location {
                            file_id,
                            range: ptr.syntax_node_ptr().text_range(),
                        })
                    }
                    Declaration::Variable { id } => {
                        source_map.expr_map_back.get(&id).map(|ptr| Location {
                            file_id,
                            range: ptr.syntax_node_ptr().text_range(),
                        })
                    }
                    Declaration::Parameter {} => None,
                    Declaration::LoadItem {} => None,
                })
                .collect(),
        );
    }
    None
}
