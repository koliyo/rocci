import http.client
import pathlib
import sys
import threading
import unittest

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))

import backend


class RenderingTests(unittest.TestCase):
    def test_datastar_patch_is_protocol_compatible(self) -> None:
        event = backend.patch_elements(backend.datastar_counter(7)).decode()
        self.assertTrue(event.startswith("event: datastar-patch-elements\n"))
        self.assertIn("data: elements   <output>7</output>\n", event)
        self.assertTrue(event.endswith("\n\n"))

    def test_htmx_counter_is_an_html_fragment(self) -> None:
        self.assertEqual(
            backend.htmx_counter(7), '<output id="htmx-counter">7</output>'
        )

    def test_python_pages_own_their_template_rendering(self) -> None:
        templates = pathlib.Path(__file__).resolve().parent / "templates"
        for name in ("datastar.html", "htmx.html"):
            source = (templates / name).read_text(encoding="utf-8")
            self.assertEqual(source.count("{{counter}}"), 1)


class HttpContractTests(unittest.TestCase):
    def setUp(self) -> None:
        root = pathlib.Path(__file__).resolve().parents[2]
        templates = pathlib.Path(__file__).resolve().parent / "templates"
        self.state = backend.State(root / "assets", templates)
        self.server = backend.ThreadingServer(("127.0.0.1", 0), backend.Handler)
        self.server.state = self.state
        self.state.host = f"127.0.0.1:{self.server.server_port}"
        self.thread = threading.Thread(target=self.server.serve_forever, daemon=True)
        self.thread.start()
        self.connection = http.client.HTTPConnection(
            "127.0.0.1", self.server.server_port, timeout=2
        )

    def tearDown(self) -> None:
        self.connection.close()
        self.server.shutdown()
        self.server.server_close()
        self.thread.join(timeout=2)

    def test_post_body_is_drained_before_connection_reuse(self) -> None:
        cookie = f"rocci_session={self.state.token}"
        headers = {
            "Cookie": cookie,
            "Origin": f"http://{self.state.host}",
            "Content-Type": "application/json",
        }
        self.connection.request(
            "POST", "/api/counter/increment", body="{}", headers=headers
        )
        response = self.connection.getresponse()
        self.assertEqual(response.status, 200)
        response.read()

        self.connection.request("GET", "/health", headers={"Cookie": cookie})
        response = self.connection.getresponse()
        self.assertEqual(response.status, 200)
        self.assertEqual(response.read(), b"ok")


if __name__ == "__main__":
    unittest.main()
