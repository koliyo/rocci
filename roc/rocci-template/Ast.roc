Ast := [].{
    ident_name = ast_ident_name
}

ast_ident_name = |ident| ident.name

Span : { start : U64, end : U64 }

Ident : { name : Str, span : Span }

AttrValue : [
    Static({ span : Span, value : Str }),
    Expr({ expr : Span }),
    Action({ name : Ident, args : Span }),
    Boolean,
]

Attr : { name : Ident, value : AttrValue, span : Span }

ComponentPath : { parts : List(Ident), roc_name : Str, span : Span }

TemplateItem : [
    Element({
        name : Ident,
        attrs : List(Attr),
        children : List(U64),
        self_closing : Bool,
        span : Span,
    }),
    ComponentCall({
        path : ComponentPath,
        attrs : List(Attr),
        children : List(U64),
        span : Span,
    }),
    Fragment({ children : List(U64), span : Span }),
    Text({ value : Str, span : Span }),
    Interpolation({ expr : Span, span : Span }),
    IfDirective({
        condition : Span,
        then_roots : List(U64),
        else_ifs : List({ condition : Span, roots : List(U64) }),
        else_roots : List(U64),
        span : Span,
    }),
    ForDirective({
        binder : Ident,
        collection : Span,
        body_roots : List(U64),
        span : Span,
    }),
    MatchDirective({
        scrutinee : Span,
        arms : List({ pattern : Span, value : U64, span : Span }),
        span : Span,
    }),
    LetDirective({ binder : Ident, expr : Span, span : Span }),
    BodyCss({ body : Span, span : Span }),
]

TemplateBlock : { nodes : List(TemplateItem), roots : List(U64), span : Span }

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

