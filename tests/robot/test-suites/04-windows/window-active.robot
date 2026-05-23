*** Settings ***
Resource    ../../resources/common.resource
Resource    ../../resources/api.resource
Resource    ../../resources/proof.resource
Suite Setup    Setup Test Environment
Suite Teardown    Teardown Test Environment
Test Tags    window    active    regression

*** Test Cases ***
Proof Created In Active Window
    [Tags]    smoke    window
    ${window}=    Create Orbital Window    start=-1h    end=+1h
    ${proof}=    Create Proof    target_node=node-active-1    window_id=${window}[id]
    Should Be Equal    ${proof}[orbital_window][id]    ${window}[id]

Proof Inactive Window Is Expired
    [Tags]    window    negative
    ${window}=    Create Orbital Window    start=-2h    end=-1h
    ${proof}=    Create Proof    target_node=node-active-2    window_id=${window}[id]
    Sleep    1s

Window Active Check Returns Status
    [Tags]    window
    ${window}=    Create Orbital Window    start=-1h    end=+1h
    ${resp}=    API Get    ${API_BASE}/api/v1/windows/${window}[id]/active
    ${body}=    Set Variable    ${resp.json()}
    Dictionary Should Contain Key    ${body}    active

Window Is Active During Period
    [Tags]    window
    ${window}=    Create Orbital Window    start=-1h    end=+1h
    ${resp}=    API Get    ${API_BASE}/api/v1/windows/${window}[id]/active
    ${body}=    Set Variable    ${resp.json()}
    Should Be True    ${body}[active]

Expired Window Is Not Active
    [Tags]    window
    ${window}=    Create Orbital Window    start=-2h    end=-1h
    ${resp}=    API Get    ${API_BASE}/api/v1/windows/${window}[id]/active
    ${body}=    Set Variable    ${resp.json()}
    Should Not Be True    ${body}[active]

Multiple Proofs In Same Window
    [Tags]    window
    ${window}=    Create Orbital Window    start=-1h    end=+1h
    ${proofs}=    Create Test Proofs In Window    ${window}[id]    count=3
    Length Should Be    ${proofs}    3
    FOR    ${proof}    IN    @{proofs}
        Should Be Equal    ${proof}[orbital_window][id]    ${window}[id]
    END

Window Overlap Detection
    [Tags]    window
    ${w1}=    Create Orbital Window    start=-2h    end=+2h
    ${w2}=    Create Orbital Window    start=-1h    end=+3h
    ${resp}=    API Post    ${API_BASE}/api/v1/windows/overlap    payload={"window_id_1": "${w1}[id]", "window_id_2": "${w2}[id]"}

Window Can Be Queried By ID
    [Tags]    window
    ${window}=    Create Orbital Window    start=-1h    end=+1h
    ${resp}=    API Get    ${API_BASE}/api/v1/windows/${window}[id]
    ${body}=    Set Variable    ${resp.json()}
    Should Be Equal    ${body}[id]    ${window}[id]

Proof Window Constraint Enforced
    [Tags]    window    negative
    Run Keyword And Expect Error    *400*    Create Proof    target_node=node-active-invalid    window_id=invalid-window-id

Window Schedule Can Be Listed
    [Tags]    window
    Create Orbital Window    start=-1h    end=+1h
    Create Orbital Window    start=-2h    end=+2h
    ${resp}=    API Get    ${API_BASE}/api/v1/windows
    Should Be True    ${resp.json().__len__()} >= 2
