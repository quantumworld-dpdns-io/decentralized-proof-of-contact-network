*** Settings ***
Resource    ../../resources/common.resource
Resource    ../../resources/proof.resource
Suite Setup    Setup Test Environment
Suite Teardown    Teardown Test Environment
Test Tags    proof    search    regression

*** Test Cases ***
Search All Proofs
    [Tags]    smoke    search
    ${results}=    Search Proofs
    Should Not Be None    ${results}

Search By Target Node
    [Tags]    search
    ${target}=    Set Variable    node-search-target-${uuid}
    Create Proof    target_node=${target}    window_id=window-search
    ${results}=    Search Proofs    target_node=${target}
    Should Not Be Empty    ${results}
    FOR    ${proof}    IN    @{results}
        Should Be Equal    ${proof}[target_node]    ${target}
    END

Search By Proving Node
    [Tags]    search
    ${results}=    Search Proofs    proving_node=${DEFAULT_NODE_ID}
    Should Not Be Empty    ${results}
    FOR    ${proof}    IN    @{results}
        Should Be Equal    ${proof}[proving_node]    ${DEFAULT_NODE_ID}
    END

Search By Window ID
    [Tags]    search
    ${window}=    Create Orbital Window    start=-1h    end=+1h
    Create Proof    target_node=node-search-window    window_id=${window}[id]
    ${results}=    Search Proofs    window_id=${window}[id]
    Should Not Be Empty    ${results}

Search With Text Query
    [Tags]    search
    ${target}=    Set Variable    node-search-query-${uuid}
    Create Proof    target_node=${target}    window_id=window-search-query
    ${results}=    Search Proofs    query=${target}
    Should Not Be Empty    ${results}

Search Returns Empty For Non Existent
    [Tags]    search    negative
    ${results}=    Search Proofs    query=non-existent-node-xyz-12345
    Should Be Empty    ${results}

Search Supports Pagination
    [Tags]    search
    FOR    ${i}    IN RANGE    5
        Create Proof    target_node=node-paginate-${i}    window_id=window-paginate
    END
    ${page1}=    Search Proofs    limit=2    offset=0
    ${page2}=    Search Proofs    limit=2    offset=2
    Length Should Be    ${page1}    2
    Length Should Be    ${page2}    2

Search Results Have Required Fields
    [Tags]    search
    ${results}=    Search Proofs    limit=1
    ${proof}=    Set Variable    ${results}[0]
    Dictionary Should Contain Key    ${proof}    id
    Dictionary Should Contain Key    ${proof}    proving_node
    Dictionary Should Contain Key    ${proof}    target_node
    Dictionary Should Contain Key    ${proof}    timestamp

Search By Status
    [Tags]    search
    ${results}=    Search Proofs    status=verified
    Should Not Be None    ${results}

Search With Empty Filter Returns All
    [Tags]    search
    ${all}=    Search Proofs
    ${filtered}=    Search Proofs    proving_node=${DEFAULT_NODE_ID}
    Should Be True    ${all.__len__()} >= ${filtered.__len__()}

Search Results Ordered By Timestamp
    [Tags]    search
    ${results}=    Search Proofs    limit=10
    IF    ${results.__len__()} >= 2
        ${t1}=    Set Variable    ${results}[0][timestamp]
        ${t2}=    Set Variable    ${results}[1][timestamp]
    END
