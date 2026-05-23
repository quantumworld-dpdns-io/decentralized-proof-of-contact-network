*** Settings ***
Resource    common.resource
Library    RequestsLibrary

*** Variables ***
${API_VERSION}    v1
${API_BASE}    /api/${API_VERSION}
${PROOFS_ENDPOINT}    ${API_BASE}/proofs
${WINDOWS_ENDPOINT}    ${API_BASE}/windows
${NODES_ENDPOINT}    ${API_BASE}/node
${NETWORK_ENDPOINT}    ${API_BASE}/network
${ANALYTICS_ENDPOINT}    ${API_BASE}/analytics
${AI_ENDPOINT}    ${API_BASE}/ai
${AUTH_ENDPOINT}    ${API_BASE}/auth
${HEALTH_ENDPOINT}    /health
${VERSION_ENDPOINT}    /version
${METRICS_ENDPOINT}    /metrics

*** Keywords ***
Create API Session
    Create Session    poi-api    ${API_BASE_URL}    headers=${AUTH_HEADER}
    ...    verify=${FALSE}

Get Auth Headers
    [Arguments]    ${token}=${DEFAULT_AUTH_TOKEN}
    &{headers}=    Create Dictionary    Authorization=Bearer ${token}    Content-Type=application/json
    RETURN    ${headers}

API Get
    [Arguments]    ${endpoint}    ${params}=${NONE}    ${expected_status}=200
    ${headers}=    Get Auth Headers
    ${resp}=    GET    ${API_BASE_URL}${endpoint}    headers=${headers}    params=${params}
    Status Should Be    ${expected_status}    ${resp}
    RETURN    ${resp}

API Post
    [Arguments]    ${endpoint}    ${payload}=${NONE}    ${expected_status}=200
    ${headers}=    Get Auth Headers
    ${resp}=    POST    ${API_BASE_URL}${endpoint}    headers=${headers}    json=${payload}
    Status Should Be    ${expected_status}    ${resp}
    RETURN    ${resp}

API Put
    [Arguments]    ${endpoint}    ${payload}=${NONE}    ${expected_status}=200
    ${headers}=    Get Auth Headers
    ${resp}=    PUT    ${API_BASE_URL}${endpoint}    headers=${headers}    json=${payload}
    Status Should Be    ${expected_status}    ${resp}
    RETURN    ${resp}

API Delete
    [Arguments]    ${endpoint}    ${expected_status}=204
    ${headers}=    Get Auth Headers
    ${resp}=    DELETE    ${API_BASE_URL}${endpoint}    headers=${headers}
    Status Should Be    ${expected_status}    ${resp}
    RETURN    ${resp}

API Patch
    [Arguments]    ${endpoint}    ${payload}=${NONE}    ${expected_status}=200
    ${headers}=    Get Auth Headers
    ${resp}=    PATCH    ${API_BASE_URL}${endpoint}    headers=${headers}    json=${payload}
    Status Should Be    ${expected_status}    ${resp}
    RETURN    ${resp}

Health Check
    ${resp}=    API Get    ${HEALTH_ENDPOINT}    expected_status=200
    ${body}=    Set Variable    ${resp.json()}
    Dictionary Should Contain Key    ${body}    status
    RETURN    ${body}

Get API Version
    ${resp}=    API Get    ${VERSION_ENDPOINT}    expected_status=200
    RETURN    ${resp.json()}

Create Proof Via API
    [Arguments]    ${target_node}    ${window_id}    ${purpose}=test    ${confidence}=1.0
    &{payload}=    Create Dictionary    target_node=${target_node}    window_id=${window_id}
    ...    purpose=${purpose}    confidence_score=${confidence}    protocol_version=${PROTOCOL_VERSION}
    ${resp}=    API Post    ${PROOFS_ENDPOINT}    payload=${payload}    expected_status=201
    RETURN    ${resp.json()}

Get Proof Via API
    [Arguments]    ${proof_id}
    ${resp}=    API Get    ${PROOFS_ENDPOINT}/${proof_id}
    RETURN    ${resp.json()}

Verify Proof Via API
    [Arguments]    ${proof_id}    ${public_key}=${NONE}
    &{params}=    Create Dictionary
    IF    ${public_key}    Set To Dictionary    ${params}    public_key=${public_key}
    ${resp}=    API Get    ${PROOFS_ENDPOINT}/${proof_id}/verify    params=${params}
    RETURN    ${resp.json()}

Search Proofs Via API
    [Arguments]    ${query}=${NONE}    ${proving_node}=${NONE}    ${target_node}=${NONE}    ${window_id}=${NONE}
    &{params}=    Create Dictionary    limit=50    offset=0
    IF    ${query}    Set To Dictionary    ${params}    q=${query}
    IF    ${proving_node}    Set To Dictionary    ${params}    proving_node=${proving_node}
    IF    ${target_node}    Set To Dictionary    ${params}    target_node=${target_node}
    IF    ${window_id}    Set To Dictionary    ${params}    window_id=${window_id}
    ${resp}=    API Get    ${PROOFS_ENDPOINT}    params=${params}
    RETURN    ${resp.json()}

Delete Proof Via API
    [Arguments]    ${proof_id}
    API Delete    ${PROOFS_ENDPOINT}/${proof_id}

Create Orbital Window Via API
    [Arguments]    ${start_time}    ${end_time}    ${window_type}=Standard
    &{payload}=    Create Dictionary    start_time=${start_time}    end_time=${end_time}    window_type=${window_type}
    ${resp}=    API Post    ${WINDOWS_ENDPOINT}    payload=${payload}    expected_status=201
    RETURN    ${resp.json()}

Get Node Info Via API
    ${resp}=    API Get    ${NODES_ENDPOINT}/info
    RETURN    ${resp.json()}

Get Node Peers Via API
    ${resp}=    API Get    ${NODES_ENDPOINT}/peers
    RETURN    ${resp.json()}

Get Node Status Via API
    ${resp}=    API Get    ${NODES_ENDPOINT}/status
    RETURN    ${resp.json()}
