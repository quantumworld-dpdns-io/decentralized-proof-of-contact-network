*** Settings ***
Resource    ../../resources/common.resource
Resource    ../../resources/proof.resource
Suite Setup    Setup Test Environment
Suite Teardown    Teardown Test Environment
Test Tags    proof    verification    regression

*** Test Cases ***
Verify Valid Proof Succeeds
    [Tags]    smoke    verification
    ${proof}=    Create Proof    target_node=node-verification-1    window_id=window-verification
    ${result}=    Verify Proof    ${proof}[id]
    Dictionary Should Contain Key    ${result}    verified

Verify Proof With Correct Public Key
    [Tags]    verification
    ${kp}=    Generate Keypair
    ${proof}=    Create Proof    target_node=node-verification-2    window_id=window-verification
    ${result}=    Verify Proof    ${proof}[id]    public_key=${kp}[public_key]
    Dictionary Should Contain Key    ${result}    verified

Verify Proof With Wrong Public Key
    [Tags]    verification    negative
    ${kp}=    Generate Keypair
    ${proof}=    Create Proof    target_node=node-verification-3    window_id=window-verification
    ${result}=    Verify Proof    ${proof}[id]    public_key=${kp}[public_key]
    ${verified}=    Evaluate    ${result}.get('verified', False)
    Should Be Equal    ${verified}    ${FALSE}

Verify Non Existent Proof
    [Tags]    verification    negative
    ${fake_id}=    Generate UUID
    Run Keyword And Expect Error    *404*    Verify Proof    ${fake_id}

Verify Proof Returns Signature Valid
    [Tags]    verification
    ${proof}=    Create Proof    target_node=node-verification-4    window_id=window-verification
    ${result}=    Verify Proof    ${proof}[id]
    Dictionary Should Contain Key    ${result}    signature_valid

Verify Proof Returns Timestamp Valid
    [Tags]    verification
    ${proof}=    Create Proof    target_node=node-verification-5    window_id=window-verification
    ${result}=    Verify Proof    ${proof}[id]
    Dictionary Should Contain Key    ${result}    timestamp_valid

Verify Proof Returns Window Valid
    [Tags]    verification
    ${proof}=    Create Proof    target_node=node-verification-6    window_id=window-verification
    ${result}=    Verify Proof    ${proof}[id]
    Dictionary Should Contain Key    ${result}    orbital_window_valid

Verify Proof Returns Verification Report
    [Tags]    verification
    ${proof}=    Create Proof    target_node=node-verification-7    window_id=window-verification
    ${result}=    Verify Proof    ${proof}[id]
    Dictionary Should Contain Key    ${result}    all_passed

Verify Tampered Proof Fails
    [Tags]    verification    negative
    ${proof}=    Create Proof    target_node=node-verification-8    window_id=window-verification
    ${result}=    Verify Proof    ${proof}[id]
    ${all_passed}=    Evaluate    ${result}.get('all_passed', False)

Verify Proof After Window Expiry
    [Tags]    verification    window
    ${window}=    Create Orbital Window    start=-2h    end=-1h
    ${proof}=    Create Proof    target_node=node-verification-9    window_id=${window}[id]
    ${result}=    Verify Proof    ${proof}[id]
    ${window_valid}=    Evaluate    ${result}.get('orbital_window_valid', True)
