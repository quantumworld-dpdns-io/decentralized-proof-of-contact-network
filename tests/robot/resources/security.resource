*** Settings ***
Resource    common.resource
Library    ../libraries/SecurityHelper.py

*** Variables ***
${MALICIOUS_ORIGIN}    https://evil.com
${SENSITIVE_PATHS}    /etc/passwd,/proc/self/environ

*** Keywords ***
Security Test Setup
    Setup Test Environment
    Set Suite Variable    ${SEC_HELPER}    SecurityHelper

Test SQL Injection On Endpoint
    [Arguments]    ${endpoint}    ${payload}=${NONE}
    ${results}=    Attempt SQL Injection    ${endpoint}    ${payload}
    FOR    ${r}    IN    @{results}
        Should Be True    ${r}[blocked]    SQL injection should be blocked: ${r}[injection]
    END

Test XSS On Endpoint
    [Arguments]    ${endpoint}    ${payload}=${NONE}
    ${results}=    Attempt XSS    ${endpoint}    ${payload}
    FOR    ${r}    IN    @{results}
        Should Be False    ${r}[reflected]    XSS payload reflected: ${r}[payload]
    END

Test Unauthorized Access
    [Arguments]    ${endpoint}    ${method}=GET
    ${results}=    Check Unauthorized Access    ${endpoint}    ${method}
    FOR    ${r}    IN    @{results}
        Should Be True    ${r}[requires_auth]    ${r}[method] ${endpoint} should require auth
    END

Test Rate Limiting
    [Arguments]    ${endpoint}    ${requests}=120
    ${result}=    Check Rate Limiting    ${endpoint}    ${requests}
    Should Be True    ${result}[rate_limited] > 0    Rate limiting should be triggered
    RETURN    ${result}

Test CORS Security
    [Arguments]    ${endpoint}
    ${result}=    Check CORS Headers    ${endpoint}    ${MALICIOUS_ORIGIN}
    Should Not Contain    ${result}[allow_origin]    ${MALICIOUS_ORIGIN}
    RETURN    ${result}

Test JWT Tampering
    [Arguments]    ${endpoint}    ${valid_token}
    ${results}=    Attempt JWT Tampering    ${endpoint}    ${valid_token}
    FOR    ${r}    IN    @{results}
        Should Be False    ${r}[authenticated]    Tampered JWT should not authenticate: ${r}[token]
    END

Test SSRF Protection
    [Arguments]    ${endpoint}
    ${results}=    Attempt SSRF    ${endpoint}
    FOR    ${r}    IN    @{results}
        Should Be True    ${r}[blocked]    SSRF target should be blocked: ${r}[target]
    END

Test Command Injection
    [Arguments]    ${endpoint}
    ${results}=    Attempt Command Injection    ${endpoint}
    FOR    ${r}    IN    @{results}
        Should Be True    ${r}[blocked]    Command injection should be blocked: ${r}[command]
    END

Test Path Traversal
    [Arguments]    ${endpoint}
    ${results}=    Attempt Path Traversal    ${endpoint}
    FOR    ${r}    IN    @{results}
        Should Be True    ${r}[blocked]    Path traversal should be blocked: ${r}[path]
    END

Test Audit Log Injection
    ${results}=    Check Audit Log Injection
    FOR    ${r}    IN    @{results}
        Should Be True    ${r}[accepted]    Normal request should be accepted
    END

Assert No Debug Endpoints Exposed
    ${results}=    Check Debug Endpoints
    FOR    ${r}    IN    @{results}
        Should Be False    ${r}[exposed]    Debug endpoint should not be exposed: ${r}[path]
    END

Assert No Default Credentials
    ${results}=    Check Default Credentials
    FOR    ${r}    IN    @{results}
        Should Be False    ${r}[success]    Default credentials should not work: ${r}[username]
    END
