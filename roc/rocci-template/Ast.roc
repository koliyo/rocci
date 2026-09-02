Ast := [].{
    ident_name = ast_ident_name
}

ast_ident_name = |ident| ident.name

Span : { start : U64, end : U64 }

Ident : { name : Str, span : Span }

TemplateBlock : { span : Span }

ComponentDecl : {
    name : Ident,
    params : Span,
    body : TemplateBlock,
    span : Span,
}

FixtureDecl : {
    name : Ident,
    target : Ident,
    value : Span,
    span : Span,
}

TestDecl : {
    name : Ident,
    value : Span,
    span : Span,
}

CssDecl : {
    body : Span,
    span : Span,
}

ModuleItem : [
    RocRegion({ span : Span }),
    Component(ComponentDecl),
    Fixture(FixtureDecl),
    Test(TestDecl),
    Css(CssDecl),
]

Document : { items : List(ModuleItem), span : Span }

Diagnostic : { code : Str, span : Span, message : Str }

ParseOutput : { document : Document, diagnostics : List(Diagnostic) }

