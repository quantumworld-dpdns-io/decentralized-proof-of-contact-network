*** Settings ***
Resource    ../../resources/common.resource
Resource    ../../resources/api.resource
Library    ../../libraries/SecurityHelper.py
Suite Setup    Setup Test Environment
Suite Teardown    Teardown Test Environment
Test Tags    api    auth    security

*** Test Cases ***
Protected Endpoint Requires Auth
    [Tags]    auth    security
    ${results}=    Check Unauthorized Access    ${PROOFS_ENDPOINT}
    FOR    ${r}    IN    @{results}
        Should Be True    ${r}[requires_auth]    ${r}[method] should require auth
    END

Valid Auth Token Succeeds
    [Tags]    auth    smoke
    ${headers}=    Get Auth Headers    ${DEFAULT_AUTH_TOKEN}
    ${resp}=    GET    ${API_BASE_URL}${PROOFS_ENDPOINT}    headers=${headers}
    Status Should Be    200    ${resp}

Invalid Auth Token Fails
    [Tags]    auth    negative
    ${headers}=    Get Auth Headers    invalid-token
    ${resp}=    GET    ${API_BASE_URL}${PROOFS_ENDPOINT}    headers=${headers}
    Status Should Be    401    ${resp}

Missing Auth Header Returns 401
    [Tags]    auth    negative
    ${headers}=    Create Dictionary    Content-Type=application/json
    ${resp}=    GET    ${API_BASE_URL}${PROOFS_ENDPOINT}    headers=${headers}
    Status Should Be    401    ${resp}

Empty Bearer Token Fails
    [Tags]    auth    negative
    ${headers}=    Get Auth Headers    ${EMPTY}
    ${resp}=    GET    ${API_BASE_URL}${PROOFS_ENDPOINT}    headers=${headers}
    Status Should Be    401    ${resp}

Auth Header Format Validated
    [Tags]    auth    security
    ${headers}=    Create Dictionary    Authorization=Basic dGVzdDpwYXNz    Content-Type=application/json
    ${resp}=    GET    ${API_BASE_URL}${PROOFS_ENDPOINT}    headers=${headers}
    Status Should Be    401    ${resp}

Health Endpoint Does Not Require Auth
    [Tags]    auth
    ${resp}=    GET    ${API_BASE_URL}/health
    Status Should Be    200    ${resp}

Token Authentication Is Consistent
    [Tags]    auth
    ${headers}=    Get Auth Headers    ${DEFAULT_AUTH_TOKEN}
    FOR    ${i}    IN RANGE    5
        ${resp}=    GET    ${API_BASE_URL}${PROOFS_ENDPOINT}    headers=${headers}
        Status Should Be    200    ${resp}
    END

Auth Token In Query Param
    [Tags]    auth
    ${resp}=    GET    ${API_BASE_URL}${PROOFS_ENDPOINT}    params={"token": "${DEFAULT_AUTH_TOKEN}"}
    Status Should Be    200    ${resp}
