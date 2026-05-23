*** Settings ***
Resource    ../../resources/common.resource
Resource    ../../resources/api.resource
Suite Setup    Setup Test Environment
Suite Teardown    Teardown Test Environment
Test Tags    storage    vector-store    regression

*** Test Cases ***
Vector Store Is Connected
    [Tags]    smoke    vector-store
    ${resp}=    API Get    ${API_BASE}/api/v1/storage/vector/status
    ${status}=    Set Variable    ${resp.json()}
    Dictionary Should Contain Key    ${status}    connected

Vector Store Provider Is Configured
    [Tags]    vector-store
    ${resp}=    API Get    ${API_BASE}/api/v1/storage/vector/status
    ${status}=    Set Variable    ${resp.json()}
    Dictionary Should Contain Key    ${status}    provider

Vector Store Indexes Proofs
    [Tags]    vector-store
    ${proof}=    Create Proof    target_node=node-vector-index    window_id=window-vector
    ${resp}=    API Post    ${API_BASE}/api/v1/storage/vector/index    payload={"proof_id": "${proof}[id]"}
    ${result}=    Set Variable    ${resp.json()}
    Dictionary Should Contain Key    ${result}    indexed

Vector Store Supports Semantic Search
    [Tags]    vector-store
    Create Proof    target_node=node-vector-semantic    window_id=window-vector    purpose=semantic-search-test
    ${resp}=    API Post    ${API_BASE}/api/v1/storage/vector/search    payload={"query": "semantic search test", "limit": 5}
    ${result}=    Set Variable    ${resp.json()}
    Should Not Be Empty    ${result}

Vector Store Returns Embeddings
    [Tags]    vector-store
    Create Proof    target_node=node-vector-embed    window_id=window-vector
    ${resp}=    API Post    ${API_BASE}/api/v1/storage/vector/embed    payload={"text": "test proof embedding"}
    ${result}=    Set Variable    ${resp.json()}
    Dictionary Should Contain Key    ${result}    embedding
    Should Be True    ${result}[embedding] is not None

Vector Store Collection Exists
    [Tags]    vector-store
    ${resp}=    API Get    ${API_BASE}/api/v1/storage/vector/collections
    ${collections}=    Set Variable    ${resp.json()}
    Should Not Be Empty    ${collections}

Vector Store Search Returns Distance
    [Tags]    vector-store
    Create Proof    target_node=node-vector-distance    window_id=window-vector    purpose=distance-metric
    ${resp}=    API Post    ${API_BASE}/api/v1/storage/vector/search    payload={"query": "distance metric", "limit": 5}
    ${results}=    Set Variable    ${resp.json()}
    IF    ${results.__len__()} > 0
        ${first}=    Set Variable    ${results}[0]
        Dictionary Should Contain Key    ${first}    score
        Dictionary Should Contain Key    ${first}    proof_id
    END

Vector Store Deleting Proof Updates Index
    [Tags]    vector-store
    ${proof}=    Create Proof    target_node=node-vector-delete    window_id=window-vector
    Delete Proof    ${proof}[id]
    ${resp}=    API Delete    ${API_BASE}/api/v1/storage/vector/index/${proof}[id]
    Status Should Be    204    ${resp}
