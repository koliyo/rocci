action = |method, uri| "@${method}('${uri}')"

Datastar := [].{
    get = |uri| action("get", uri)
    post = |uri| action("post", uri)
    put = |uri| action("put", uri)
    patch = |uri| action("patch", uri)
    delete = |uri| action("delete", uri)

    get_with = |uri, _opts| action("get", uri)
    post_with = |uri, _opts| action("post", uri)
    put_with = |uri, _opts| action("put", uri)
    patch_with = |uri, _opts| action("patch", uri)
    delete_with = |uri, _opts| action("delete", uri)
}
