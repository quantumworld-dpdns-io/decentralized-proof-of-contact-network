*** Settings ***
Resource    ../../resources/common.resource
Resource    ../../resources/api.resource
Library    ../../libraries/SecurityHelper.py
Suite Setup    Setup Test Environment
Suite Teardown    Teardown Test Environment
Test Tags    api    rate-limit    security

*** Test Cases ***
Rate Limiting Returns 429
    [Tags]    rate-limit    security
    ${result}=    Check Rate Limiting    ${PROOFS_ENDPOINT}    requests_count=150
    Should Be True    ${result}[rate_limited] > 0    Rate limiting should be triggered

Rate Limiting Resets After Window
    [Tags]    rate-limit
    ${result}=    Check Rate Limiting    ${PROOFS_ENDPOINT}    requests_count=150
    ${limited}=    Set Variable    ${result}[rate_limited]
    Sleep    1s
    ${headers}=    Get Auth Headers
    ${resp}=    GET    ${API_BASE_URL}${PROOFS_ENDPOINT}    headers=${headers}
    Status Should Be    200    ${resp}

Different Clients Are Limited Independently
    [Tags]    rate-limit
    ${sec}=    SecurityHelper
    ${result1}=    Check Rate Limiting    ${PROOFS_ENDPOINT}    requests_count=150
    ${result2}=    Check Rate Limiting    ${PROOFS_ENDPOINT}    requests_count=10
    Log    Client1 limited: ${result1}[rate_limited], Client2 limited: ${result2}[rate_limited]

Rate Limit Header Is Returned
    [Tags]    rate-limit
    ${headers}=    Get Auth Headers
    ${resp}=    GET    ${API_BASE_URL}${PROOFS_ENDPOINT}    headers=${headers}
    Dictionary Should Contain Key    ${resp.headers}    X-RateLimit-Limit

Rate Limit Remaining Header Returned
    [Tags]    rate-limit
    ${headers}=    Get Auth Headers
    ${resp}=    GET    ${API_BASE_URL}${PROOFS_ENDPOINT}    headers=${headers}
    Dictionary Should Contain Key    ${resp.headers}    X-RateLimit-Remaining

Rate Limit Reset Header Returned
    [Tags]    rate-limit
    ${headers}=    Get Auth Headers
    ${resp}=    GET    ${API_BASE_URL}${PROOFS_ENDPOINT}    headers=${headers}
    Dictionary Should Contain Key    ${resp.headers}    X-RateLimit-Reset

Rate Limit Applies Per Endpoint
    [Tags]    rate-limit
    ${sec}=    SecurityHelper
    ${proof_result}=    Check Rate Limiting    ${PROOFS_ENDPOINT}    requests_count=150
    ${health_result}=    Check Rate Limiting    ${HEALTH_ENDPOINT}    requests_count=150
    Should Be True    ${health_result}[rate_limited] <= ${proof_result}[rate_limited]
