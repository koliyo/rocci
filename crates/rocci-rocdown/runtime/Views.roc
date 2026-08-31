# Field names match crates/rocci-ui/src/view.rs.

Views := [].{
    SiteView : {
        title : Str,
        description : Str,
        base_url : Str,
        language : Str,
        repository : Str,
        social_image : Str,
        favicon : Str,
        apple_touch_icon : Str,
        subtitle : Str,
        footer : Str,
    }

    LaneView : {
        label : Str,
        href : Str,
        current : Bool,
    }

    NavItemView : {
        title : Str,
        href : Str,
        class_name : Str,
    }

    NavGroupView : {
        title : Str,
        href : Str,
        open : Bool,
        items : List(NavItemView),
        children : List(NavGroupView),
    }

    BreadcrumbView : {
        title : Str,
        href : Str,
    }

    OutlineView : {
        id : Str,
        title : Str,
        level : Str,
    }

    ResourceView : {
        stylesheet : Str,
        csp : Str,
        canonical : Str,
        module_script : Str,
        chrome_script : Str,
        playground_css : Str,
        playground_session : Str,
    }

    CollectionItemView : {
        route : Str,
        title : Str,
        summary : Str,
        published : Str,
        updated : Str,
        authors : List(Str),
        tags : List(Str),
    }

    PageView : {
        site : SiteView,
        lanes : List(LaneView),
        sidebar : List(NavGroupView),
        route : Str,
        title : Str,
        document_title : Str,
        description : Str,
        layout : Str,
        published : Str,
        updated : Str,
        authors : List(Str),
        tags : List(Str),
        collection : Str,
        collection_items : List(CollectionItemView),
        outline : List(OutlineView),
        breadcrumbs : List(BreadcrumbView),
        previous : NavItemView,
        next : NavItemView,
        resources : ResourceView,
    }

    Page a : {
        article_path : Str,
        output_path : Str,
        segments : List(a),
        view : PageView,
    }
}
