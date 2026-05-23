*** Settings ***
Resource    ../../resources/common.resource
Resource    ../../resources/ai.resource
Resource    ../../resources/proof.resource
Suite Setup    Setup Test Environment
Suite Teardown    Teardown Test Environment
Test Tags    ai    nl-query    regression

*** Test Cases ***
Natural Language Query Returns Results
    [Tags]    smoke    ai
    Create Proof    target_node=node-nl-query    window_id=window-nl
    ${results}=    NL Query Should Return Results    show me all proofs    minimum_results=1

Query By Node ID
    [Tags]    ai
    Create Proof    target_node=node-nl-specific    window_id=window-nl-specific
    ${results}=    Query Proofs With Natural Language    proofs involving node-nl-specific
    Should Not Be Empty    ${results}

Query By Window
    [Tags]    ai
    ${window}=    Create Orbital Window    start=-1h    end=+1h
    Create Proof    target_node=node-nl-window    window_id=${window}[id]
    ${results}=    Query Proofs With Natural Language    proofs in window ${window}[id]
    Should Not Be Empty    ${results}

Query Counts Proofs
    [Tags]    ai
    FOR    ${i}    IN RANGE    3
        Create Proof    target_node=node-nl-count-${i}    window_id=window-nl-count
    END
    ${results}=    Query Proofs With Natural Language    how many proofs exist
    Should Not Be Empty    ${results}

Query For Recent Proofs
    [Tags]    ai
    Create Proof    target_node=node-nl-recent    window_id=window-nl-recent
    ${results}=    Query Proofs With Natural Language    most recent proofs    limit=5
    Should Not Be Empty    ${results}

Query With Invalid Input
    [Tags]    ai    negative
    ${results}=    Query Proofs With Natural Language    ${EMPTY}
    Should Be Empty    ${results}

Query Returns Structured Data
    [Tags]    ai
    Create Proof    target_node=node-nl-structured    window_id=window-nl-structured
    ${results}=    Query Proofs With Natural Language    list all proofs
    ${first}=    Set Variable    ${results}[0]
    Dictionary Should Contain Key    ${first}    id

NL Query Supports Time Range
    [Tags]    ai
    ${results}=    Query Proofs With Natural Language    proofs from the last 24 hours
    Should Not Be None    ${results}

Query By Proof Purpose
    [Tags]    ai
    Create Proof    target_node=node-nl-purpose    window_id=window-nl-purpose    purpose=test-purpose-xyz
    ${results}=    Query Proofs With Natural Language    proofs with purpose test-purpose-xyz
    Should Not Be Empty    ${results}

NL Query Handles Complex Questions
    [Tags]    ai
    Create Proof    target_node=node-nl-complex    window_id=window-nl-complex
    ${results}=    Query Proofs With Natural Language    which node has the most proofs
    Should Not Be None    ${results}
