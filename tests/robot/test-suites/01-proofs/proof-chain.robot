*** Settings ***
Resource    ../../resources/common.resource
Resource    ../../resources/proof.resource
Suite Setup    Setup Test Environment
Suite Teardown    Teardown Test Environment
Test Tags    proof    chain    regression

*** Test Cases ***
Create Single Proof Chain
    [Tags]    smoke    chain
    ${chain}=    Create Proof Chain    count=1
    Length Should Be    ${chain}    1

Create Multi Proof Chain
    [Tags]    chain
    ${chain}=    Create Proof Chain    count=5
    Length Should Be    ${chain}    5

Chain Elements Have Sequential Positions
    [Tags]    chain
    ${chain}=    Create Proof Chain    count=3
    FOR    ${i}    IN RANGE    ${chain.__len__()}
        ${pos}=    Set Variable    ${chain}[${i}][metadata][chain_position]
        Should Be Equal As Integers    ${pos}    ${i}
    END

Get Proof Chain By First Proof
    [Tags]    chain
    ${chain}=    Create Proof Chain    count=3
    ${first_id}=    Set Variable    ${chain}[0][id]
    ${retrieved}=    Get Proof Chain    ${first_id}
    Length Should Be    ${retrieved}    3

Verify Chain Integrity Valid
    [Tags]    chain
    ${chain}=    Create Proof Chain    count=3
    ${first_id}=    Set Variable    ${chain}[0][id]
    ${result}=    Verify Chain Integrity    ${first_id}
    Dictionary Should Contain Key    ${result}    valid
    Should Be True    ${result}[valid]

Chain Merkle Root Is Consistent
    [Tags]    chain
    ${chain}=    Create Proof Chain    count=3
    ${first_id}=    Set Variable    ${chain}[0][id]
    ${result1}=    Verify Chain Integrity    ${first_id}
    ${result2}=    Verify Chain Integrity    ${first_id}
    Should Be Equal    ${result1}[merkle_root]    ${result2}[merkle_root]

Empty Chain Is Not Valid
    [Tags]    chain    negative
    ${fake_id}=    Generate UUID
    Run Keyword And Expect Error    *404*    Verify Chain Integrity    ${fake_id}

Chain With Single Proof Is Valid
    [Tags]    chain
    ${chain}=    Create Proof Chain    count=1
    ${first_id}=    Set Variable    ${chain}[0][id]
    ${result}=    Verify Chain Integrity    ${first_id}
    Should Be True    ${result}[valid]

Chain Proofs Share Window
    [Tags]    chain
    ${window}=    Create Orbital Window    start=-1h    end=+1h
    ${chain}=    Create Proof Chain    count=3
    FOR    ${proof}    IN    @{chain}
        Should Be Equal    ${proof}[orbital_window][id]    ${window}[id]
    END

Chain Depth Increases With Append
    [Tags]    chain
    ${chain}=    Create Proof Chain    count=3
    ${max_pos}=    Evaluate    max(p['metadata']['chain_position'] for p in ${chain})
    Should Be Equal As Integers    ${max_pos}    2
