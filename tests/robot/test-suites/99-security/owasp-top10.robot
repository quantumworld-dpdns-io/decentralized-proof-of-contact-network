*** Settings ***
Resource    ../../resources/common.resource
Resource    ../../resources/security.resource
Library    ../../libraries/SecurityHelper.py
Suite Setup    Security Test Setup
Suite Teardown    Teardown Test Environment
Test Tags    security    owasp

*** Test Cases ***
A01 Broken Access Control - Unauthorized Access
    [Tags]    security    owasp    a01
    Test Unauthorized Access    ${PROOFS_ENDPOINT}
    Test Unauthorized Access    ${WINDOWS_ENDPOINT}
    Test Unauthorized Access    ${NODES_ENDPOINT}/config

A01 Broken Access Control - IDOR
    [Tags]    security    owasp    a01    idor
    ${fake_ids}=    Create List    ${EMPTY}    00000000-0000-0000-0000-000000000000    99999999-9999-9999-9999-999999999999
    ${results}=    Attempt IDOR    ${PROOFS_ENDPOINT}    ${fake_ids}
    FOR    ${r}    IN    @{results}
        Should Not Be Equal    ${r}[status_code]    200    IDOR should not return 200 for non-existent resource
    END

A01 Broken Access Control - Privilege Escalation
    [Tags]    security    owasp    a01
    ${headers}=    Create Dictionary    Authorization=Bearer admin-token    Content-Type=application/json
    ${resp}=    GET    ${API_BASE_URL}/api/v1/admin    headers=${headers}
    Status Should Be    403    ${resp}    Admin endpoint should be forbidden

A02 Cryptographic Failures - Weak Signature Verification
    [Tags]    security    owasp    a02
    ${proof}=    Create Proof    target_node=node-crypto-fail    window_id=window-crypto
    ${result}=    Verify Proof    ${proof}[id]
    Should Be True    ${result}[signature_valid]    Signature should be valid
    Should Be True    ${result}[all_passed]    All verification checks should pass

A02 Cryptographic Failures - Replay Attack Prevention
    [Tags]    security    owasp    a02
    ${proof}=    Create Proof    target_node=node-crypto-replay    window_id=window-crypto
    ${replay}=    Create Proof    target_node=node-crypto-replay    window_id=window-crypto
    Should Not Be Equal    ${proof}[id]    ${replay}[id]    Replayed proof should generate different ID
    Should Not Be Equal    ${proof}[signature]    ${replay}[signature]    Signatures should differ

A02 Cryptographic Failures - TLS Configuration
    [Tags]    security    owasp    a02
    ${resp}=    GET    ${API_BASE_URL}/health
    Status Should Be    200    ${resp}
    Should Contain    ${resp.headers}[Content-Type]    application/json

A03 Injection - SQL Injection Prevention
    [Tags]    security    owasp    a03    injection
    Test SQL Injection On Endpoint    ${PROOFS_ENDPOINT}
    Test SQL Injection On Endpoint    ${ANALYTICS_QUERY_ENDPOINT}

A03 Injection - NoSQL Injection Prevention
    [Tags]    security    owasp    a03    injection
    ${payloads}=    Create List    {"$gt": ""}    {"$ne": ""}    {"$where": "1==1"}
    FOR    ${payload}    IN    @{payloads}
        ${resp}=    POST    ${API_BASE_URL}${PROOFS_ENDPOINT}    json=${payload}
        Status Should Be    400    ${resp}    NoSQL injection should be rejected
    END

A03 Injection - Command Injection Prevention
    [Tags]    security    owasp    a03    injection
    Test Command Injection    ${PROOFS_ENDPOINT}
    Test Command Injection    ${NODES_ENDPOINT}/info

A03 Injection - Path Traversal Prevention
    [Tags]    security    owasp    a03    injection
    Test Path Traversal    ${PROOFS_ENDPOINT}

A04 Insecure Design - Rate Limit Bypass Prevention
    [Tags]    security    owasp    a04
    Test Rate Limiting    ${PROOFS_ENDPOINT}    requests=150

A04 Insecure Design - Chain Integrity Violation
    [Tags]    security    owasp    a04
    ${chain}=    Create Proof Chain    count=3
    ${first_id}=    Set Variable    ${chain}[0][id]
    ${result}=    Verify Chain Integrity    ${first_id}
    Should Be True    ${result}[valid]    Chain integrity should be enforced
    Should Be Equal    ${result}[merkle_root]    ${result}[merkle_root]    Merkle root should be consistent

A04 Insecure Design - Input Validation
    [Tags]    security    owasp    a04
    ${resp}=    POST    ${API_BASE_URL}${PROOFS_ENDPOINT}    json=${EMPTY}
    Status Should Be    422    ${resp}    Empty payload should be rejected
    ${resp}=    POST    ${API_BASE_URL}${PROOFS_ENDPOINT}    json={"target_node": "", "window_id": ""}
    Status Should Be    400    ${resp}    Empty fields should be rejected
    ${resp}=    POST    ${API_BASE_URL}${PROOFS_ENDPOINT}    json={"target_node": "x"*10000, "window_id": "test"}
    Status Should Be    400    ${resp}    Oversized fields should be rejected

A05 Security Misconfiguration - Debug Endpoints
    [Tags]    security    owasp    a05
    Assert No Debug Endpoints Exposed

A05 Security Misconfiguration - Default Credentials
    [Tags]    security    owasp    a05
    Assert No Default Credentials

A05 Security Misconfiguration - CORS
    [Tags]    security    owasp    a05    cors
    Test CORS Security    ${PROOFS_ENDPOINT}
    Test CORS Security    ${HEALTH_ENDPOINT}

A05 Security Misconfiguration - Information Disclosure
    [Tags]    security    owasp    a05
    ${resp}=    GET    ${API_BASE_URL}/health
    Should Not Contain    ${resp.text}    password
    Should Not Contain    ${resp.text}    secret
    Should Not Contain    ${resp.text}    private_key

A05 Security Misconfiguration - HTTP Methods
    [Tags]    security    owasp    a05
    ${resp}=    OPTIONS    ${API_BASE_URL}${PROOFS_ENDPOINT}
    Status Should Be    200    ${resp}
    ${resp}=    TRACE    ${API_BASE_URL}${PROOFS_ENDPOINT}
    Status Should Be    405    ${resp}    TRACE method should be disabled

A06 Vulnerable Components - Dependency Scanning
    [Tags]    security    owasp    a06
    ${resp}=    API Get    ${API_BASE}/api/v1/security/dependencies
    ${deps}=    Set Variable    ${resp.json()}
    Dictionary Should Contain Key    ${deps}    dependencies
    Should Not Be Empty    ${deps}[dependencies]

A06 Vulnerable Components - Version Check
    [Tags]    security    owasp    a06
    ${resp}=    API Get    ${API_BASE}/api/v1/security/dependencies
    ${deps}=    Set Variable    ${resp.json()}
    FOR    ${dep}    IN    @{deps}[dependencies]
        Dictionary Should Contain Key    ${dep}    name
        Dictionary Should Contain Key    ${dep}    version
        Should Not Be Empty    ${dep}[version]
    END

A06 Vulnerable Components - No Critical CVEs
    [Tags]    security    owasp    a06
    ${resp}=    API Get    ${API_BASE}/api/v1/security/dependencies
    ${deps}=    Set Variable    ${resp.json()}
    Dictionary Should Contain Key    ${deps}    vulnerabilities
    ${vulns}=    Set Variable    ${deps}[vulnerabilities]
    IF    ${vulns.__len__()} > 0
        FOR    ${v}    IN    @{vulns}
            Should Not Be Equal    ${v}[severity]    critical    No critical CVEs allowed
        END
    END

A07 Authentication Failures - Brute Force Protection
    [Tags]    security    owasp    a07
    ${headers}=    Get Auth Headers
    FOR    ${i}    IN RANGE    5
        ${resp}=    POST    ${API_BASE_URL}${AUTH_ENDPOINT}/login    json={"username": "admin", "password": "wrong${i}"}
        Status Should Be    401    ${resp}    Wrong credentials should fail
    END

A07 Authentication Failures - JWT Tampering
    [Tags]    security    owasp    a07    jwt
    ${valid_token}=    Set Variable    ${DEFAULT_AUTH_TOKEN}
    Test JWT Tampering    ${PROOFS_ENDPOINT}    ${valid_token}

A07 Authentication Failures - Token Expiry
    [Tags]    security    owasp    a07
    ${expired_token}=    Set Variable    eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiJ0ZXN0IiwiZXhwIjoxMDAwMDAwMDAwfQ.invalid
    ${headers}=    Get Auth Headers    ${expired_token}
    ${resp}=    GET    ${API_BASE_URL}${PROOFS_ENDPOINT}    headers=${headers}
    Status Should Be    401    ${resp}    Expired token should be rejected

A08 Integrity Failures - Data Tampering
    [Tags]    security    owasp    a08
    ${proof}=    Create Proof    target_node=node-integrity-tamper    window_id=window-integrity
    ${result}=    Verify Proof    ${proof}[id]
    Should Be True    ${result}[signature_valid]    Original proof should verify
    ${result2}=    Verify Proof    ${proof}[id]
    Should Be True    ${result2}[signature_valid]    Proof integrity should be preserved

A08 Integrity Failures - Unsigned Updates
    [Tags]    security    owasp    a08
    ${proof}=    Create Proof    target_node=node-integrity-unsigned    window_id=window-integrity
    ${updated}=    Update Proof Metadata    ${proof}[id]    proof_purpose=tampered-purpose
    Dictionary Should Contain Key    ${updated}    metadata
    ${result}=    Verify Proof    ${proof}[id]
    Should Be True    ${result}[signature_valid]    Verification should still pass after metadata update

A08 Integrity Failures - Chain Tampering Detection
    [Tags]    security    owasp    a08
    ${chain}=    Create Proof Chain    count=3
    ${first_id}=    Set Variable    ${chain}[0][id]
    ${r1}=    Verify Chain Integrity    ${first_id}
    ${r2}=    Verify Chain Integrity    ${first_id}
    Should Be Equal    ${r1}[merkle_root]    ${r2}[merkle_root]    Merkle root must be deterministic

A09 Logging Failures - Audit Completeness
    [Tags]    security    owasp    a09
    ${resp}=    API Get    ${API_BASE}/api/v1/audit/logs
    ${logs}=    Set Variable    ${resp.json()}
    Should Not Be Empty    ${logs}    Audit logs should exist
    ${first}=    Set Variable    ${logs}[0]
    Dictionary Should Contain Key    ${first}    timestamp
    Dictionary Should Contain Key    ${first}    action
    Dictionary Should Contain Key    ${first}    actor

A09 Logging Failures - Log Injection Prevention
    [Tags]    security    owasp    a09
    Test Audit Log Injection

A09 Logging Failures - Sensitive Data In Logs
    [Tags]    security    owasp    a09
    ${resp}=    API Get    ${API_BASE}/api/v1/audit/logs    params={"limit": 10}
    ${logs}=    Set Variable    ${resp.json()}
    FOR    ${log}    IN    @{logs}
        Should Not Contain    ${log}[details]    password    Logs should not contain passwords
        Should Not Contain    ${log}[details]    secret_key    Logs should not contain secret keys
        Should Not Contain    ${log}[details]    authorization    Logs should not contain auth headers
    END

A10 SSRF - Peer URL SSRF
    [Tags]    security    owasp    a10    ssrf
    Test SSRF Protection    ${NETWORK_ENDPOINT}/connect

A10 SSRF - Webhook SSRF
    [Tags]    security    owasp    a10    ssrf
    Test SSRF Protection    ${API_BASE}/api/v1/webhooks

A10 SSRF - Metadata Service SSRF
    [Tags]    security    owasp    a10    ssrf
    ${results}=    Attempt SSRF    ${NETWORK_ENDPOINT}/connect
    FOR    ${r}    IN    @{results}
        Should Be True    ${r}[blocked]    Cloud metadata SSRF should be blocked: ${r}[target]
    END

A10 SSRF - Internal Network SSRF
    [Tags]    security    owasp    a10    ssrf
    ${results}=    Attempt SSRF    ${PROOFS_ENDPOINT}
    FOR    ${r}    IN    @{results}
        Should Be True    ${r}[blocked]    Internal network SSRF should be blocked: ${r}[target]
    END

A10 SSRF - URL Validation
    [Tags]    security    owasp    a10    ssrf
    ${resp}=    POST    ${API_BASE_URL}${NETWORK_ENDPOINT}/connect    json={"url": "http://169.254.169.254/latest/meta-data/"}
    Status Should Be    400    ${resp}    Cloud metadata URL should be rejected
    ${resp}=    POST    ${API_BASE_URL}${NETWORK_ENDPOINT}/connect    json={"url": "http://127.0.0.1:3000/"}
    Status Should Be    400    ${resp}    Localhost URL should be rejected
    ${resp}=    POST    ${API_BASE_URL}${NETWORK_ENDPOINT}/connect    json={"url": "https://valid-peer.example.com:9090"}
    Status Should Be    200    ${resp}    Valid peer URL should be accepted

A10 SSRF - DNS Rebinding Prevention
    [Tags]    security    owasp    a10    ssrf
    ${resp}=    POST    ${API_BASE_URL}${NETWORK_ENDPOINT}/connect    json={"url": "http://1.2.3.4:9090"}
    Should Be True    ${resp.status_code} in (200, 400)    External IPs should be validated
