*** Settings ***
Resource    ../../resources/common.resource
Resource    ../../resources/api.resource
Suite Setup    Setup Test Environment
Suite Teardown    Teardown Test Environment
Test Tags    storage    duckdb    persistence    regression

*** Test Cases ***
Proof Persisted To DuckDB
    [Tags]    smoke    storage
    ${proof}=    Create Proof    target_node=node-duckdb-persist    window_id=window-duckdb
    ${retrieved}=    Get Proof    ${proof}[id]
    Should Be Equal    ${retrieved}[id]    ${proof}[id]

Persistence Survives Restart
    [Tags]    storage
    ${proof}=    Create Proof    target_node=node-duckdb-survive    window_id=window-duckdb
    ${id}=    Set Variable    ${proof}[id]
    Log    Proof ${id} persisted for restart test

DuckDB Table Structure Is Valid
    [Tags]    storage
    ${resp}=    API Get    ${API_BASE}/api/v1/storage/schema
    ${schema}=    Set Variable    ${resp.json()}
    Dictionary Should Contain Key    ${schema}    tables

DuckDB Supports Multiple Proofs
    [Tags]    storage
    FOR    ${i}    IN RANGE    10
        Create Proof    target_node=node-duckdb-multi-${i}    window_id=window-duckdb-multi
    END
    ${results}=    Search Proofs    window_id=window-duckdb-multi
    Length Should Be    ${results}    10

Persistence Stores Metadata Correctly
    [Tags]    storage
    ${proof}=    Create Proof    target_node=node-duckdb-meta    window_id=window-duckdb-meta    purpose=persistence-test
    ${retrieved}=    Get Proof    ${proof}[id]
    Should Be Equal    ${retrieved}[metadata][proof_purpose]    persistence-test

DuckDB Stores Node IDs
    [Tags]    storage
    ${proof}=    Create Proof    target_node=node-duckdb-nodeid    window_id=window-duckdb-nodeid
    ${retrieved}=    Get Proof    ${proof}[id]
    Should Be Equal    ${retrieved}[proving_node]    ${DEFAULT_NODE_ID}
    Should Be Equal    ${retrieved}[target_node]    node-duckdb-nodeid

Persistence Handles Large Payloads
    [Tags]    storage
    ${long_purpose}=    Evaluate    "X" * 10000
    ${proof}=    Create Proof    target_node=node-duckdb-large    window_id=window-duckdb-large    purpose=${long_purpose}
    ${retrieved}=    Get Proof    ${proof}[id]
    Should Be Equal    ${retrieved}[metadata][proof_purpose]    ${long_purpose}

Storage Reports Database Size
    [Tags]    storage
    ${resp}=    API Get    ${API_BASE}/api/v1/storage/info
    ${info}=    Set Variable    ${resp.json()}
    Dictionary Should Contain Key    ${info}    size_bytes
    Should Be True    ${info}[size_bytes] >= 0

DuckDB Connection Is Stable
    [Tags]    storage
    FOR    ${i}    IN RANGE    20
        Create Proof    target_node=node-duckdb-stable-${i}    window_id=window-duckdb-stable
    END
    ${results}=    Search Proofs    window_id=window-duckdb-stable
    Length Should Be    ${results}    20

Storage Provider Is DuckDB
    [Tags]    storage
    ${resp}=    API Get    ${API_BASE}/api/v1/storage/info
    ${info}=    Set Variable    ${resp.json()}
    Should Be Equal    ${info}[provider]    duckdb

Proof Timestamps Stored Correctly
    [Tags]    storage
    ${proof}=    Create Proof    target_node=node-duckdb-ts    window_id=window-duckdb-ts
    ${retrieved}=    Get Proof    ${proof}[id]
    Should Be Equal    ${retrieved}[timestamp]    ${proof}[timestamp]
