*** Settings ***
Resource    ../../resources/common.resource
Resource    ../../resources/api.resource
Suite Setup    Setup Test Environment
Suite Teardown    Teardown Test Environment
Test Tags    api    rest    regression

*** Test Cases ***
API Root Returns Info
    [Tags]    smoke    api
    ${resp}=    API Get    /api
    ${body}=    Set Variable    ${resp.json()}
    Dictionary Should Contain Key    ${body}    name
    Dictionary Should Contain Key    ${body}    version

API V1 Endpoints Exist
    [Tags]    api
    ${resp}=    API Get    ${API_BASE}
    ${body}=    Set Variable    ${resp.json()}
    Dictionary Should Contain Key    ${body}    endpoints

Proofs Endpoint Returns List
    [Tags]    api
    ${resp}=    API Get    ${PROOFS_ENDPOINT}
    ${body}=    Set Variable    ${resp.json()}

Create Proof Via POST
    [Tags]    api
    ${resp}=    API Post    ${PROOFS_ENDPOINT}    payload={"target_node": "node-api-test", "window_id": "window-api-test"}
    Status Should Be    201    ${resp}

Get Proof By ID
    [Tags]    api
    ${proof}=    Create Proof    target_node=node-api-get    window_id=window-api-get
    ${resp}=    API Get    ${PROOFS_ENDPOINT}/${proof}[id]
    ${body}=    Set Variable    ${resp.json()}
    Should Be Equal    ${body}[id]    ${proof}[id]

Update Proof Via PATCH
    [Tags]    api
    ${proof}=    Create Proof    target_node=node-api-patch    window_id=window-api-patch
    ${resp}=    API Patch    ${PROOFS_ENDPOINT}/${proof}[id]    payload={"purpose": "updated-via-patch"}
    ${body}=    Set Variable    ${resp.json()}
    Should Be Equal    ${body}[metadata][proof_purpose]    updated-via-patch

Delete Proof Via DELETE
    [Tags]    api
    ${proof}=    Create Proof    target_node=node-api-delete    window_id=window-api-delete
    ${resp}=    API Delete    ${PROOFS_ENDPOINT}/${proof}[id]
    Status Should Be    204    ${resp}

List Windows Endpoint
    [Tags]    api
    ${resp}=    API Get    ${WINDOWS_ENDPOINT}
    ${body}=    Set Variable    ${resp.json()}

Node Info Endpoint
    [Tags]    api
    ${resp}=    API Get    ${NODES_ENDPOINT}/info
    ${body}=    Set Variable    ${resp.json()}
    Dictionary Should Contain Key    ${body}    node_id

API Returns Correct Status Codes
    [Tags]    api
    ${resp}=    API Get    /nonexistent    expected_status=404
    ${resp}=    API Post    ${PROOFS_ENDPOINT}    payload={}    expected_status=422

API Supports Pagination Parameters
    [Tags]    api
    FOR    ${i}    IN RANGE    5
        Create Proof    target_node=node-api-page-${i}    window_id=window-api-page
    END
    ${resp}=    API Get    ${PROOFS_ENDPOINT}    params={"limit": 2, "offset": 0}
    ${body}=    Set Variable    ${resp.json()}
    Length Should Be    ${body}    2

API Content Type Is JSON
    [Tags]    api
    ${resp}=    API Get    ${HEALTH_ENDPOINT}
    Should Contain    ${resp.headers}[Content-Type]    application/json

API Headers Include Request ID
    [Tags]    api
    ${resp}=    API Get    ${HEALTH_ENDPOINT}
    Dictionary Should Contain Key    ${resp.headers}    X-Request-ID
