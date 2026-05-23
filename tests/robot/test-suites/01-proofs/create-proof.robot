*** Settings ***
Resource    ../../resources/common.resource
Resource    ../../resources/proof.resource
Suite Setup    Setup Test Environment
Suite Teardown    Teardown Test Environment
Test Tags    proof    regression

*** Test Cases ***
Create Valid Proof
    [Tags]    smoke    proof
    ${proof}=    Create Proof    target_node=node-002    window_id=window-001
    Should Not Be Empty    ${proof}[id]
    Should Be Equal    ${proof}[proving_node]    ${DEFAULT_NODE_ID}

Create Proof With Custom Target
    [Tags]    proof
    ${target}=    Set Variable    node-custom-${uuid}
    ${proof}=    Create Proof    target_node=${target}    window_id=window-002
    Should Be Equal    ${proof}[target_node]    ${target}

Create Proof With Metadata
    [Tags]    proof
    ${proof}=    Create Proof    target_node=node-003    window_id=window-003    purpose=meetup-confirmation    confidence_score=0.95
    Should Be Equal    ${proof}[metadata][proof_purpose]    meetup-confirmation
    Should Be Equal As Numbers    ${proof}[metadata][confidence_score]    0.95

Create Proof With Empty Target
    [Tags]    proof    negative
    Run Keyword And Expect Error    *400*    Create Proof    target_node=${EMPTY}    window_id=window-001

Create Proof With Empty Window
    [Tags]    proof    negative
    Run Keyword And Expect Error    *400*    Create Proof    target_node=node-002    window_id=${EMPTY}

Create Proof During Active Window
    [Tags]    proof    window
    ${window}=    Create Orbital Window    start=-1h    end=+1h
    ${proof}=    Create Proof    target_node=node-004    window_id=${window}[id]
    Should Be Equal    ${proof}[orbital_window][id]    ${window}[id]

Create Proof With Invalid Confidence Score
    [Tags]    proof    negative
    Run Keyword And Expect Error    *400*    Create Proof    target_node=node-005    window_id=window-001    confidence_score=1.5

Create Proof With Negative Confidence Score
    [Tags]    proof    negative
    Run Keyword And Expect Error    *400*    Create Proof    target_node=node-006    window_id=window-001    confidence_score=-0.5

Create Proof Returns Timestamp
    [Tags]    proof
    ${proof}=    Create Proof    target_node=node-007    window_id=window-001
    Dictionary Should Contain Key    ${proof}    timestamp
    Should Not Be Empty    ${proof}[timestamp]

Create Proof Has Protocol Version
    [Tags]    proof
    ${proof}=    Create Proof    target_node=node-008    window_id=window-001
    Should Be Equal    ${proof}[metadata][protocol_version]    ${PROTOCOL_VERSION}

Create Multiple Proofs Sequentially
    [Tags]    proof
    ${ids}=    Create List
    FOR    ${i}    IN RANGE    5
        ${proof}=    Create Proof    target_node=node-batch-${i}    window_id=window-batch
        Append To List    ${ids}    ${proof}[id]
    END
    Length Should Be    ${ids}    5

Create Proof With Long Purpose String
    [Tags]    proof
    ${long_purpose}=    Evaluate    "A" * 1000
    ${proof}=    Create Proof    target_node=node-009    window_id=window-001    purpose=${long_purpose}
    Should Be Equal    ${proof}[metadata][proof_purpose]    ${long_purpose}

Create Proof Returns Signature
    [Tags]    proof
    ${proof}=    Create Proof    target_node=node-010    window_id=window-001
    Dictionary Should Contain Key    ${proof}    signature

Create Proof Generates Unique IDs
    [Tags]    proof
    ${p1}=    Create Proof    target_node=node-011    window_id=window-001
    ${p2}=    Create Proof    target_node=node-011    window_id=window-001
    Should Not Be Equal    ${p1}[id]    ${p2}[id]
