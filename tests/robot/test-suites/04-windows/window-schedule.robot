*** Settings ***
Resource    ../../resources/common.resource
Resource    ../../resources/api.resource
Suite Setup    Setup Test Environment
Suite Teardown    Teardown Test Environment
Test Tags    window    schedule    regression

*** Test Cases ***
Create Standard Window
    [Tags]    smoke    window
    ${window}=    Create Orbital Window    start=-1h    end=+1h
    Should Be Valid Window    ${window}
    Should Be Equal    ${window}[window_type]    Standard

Create Extended Window
    [Tags]    window
    ${window}=    Create Orbital Window    start=-1h    end=+1h    window_type=Extended
    Should Be Equal    ${window}[window_type]    Extended

Create Emergency Window
    [Tags]    window
    ${window}=    Create Orbital Window    start=-30m    end=+30m    window_type=Emergency
    Should Be Equal    ${window}[window_type]    Emergency

Window Duration Is Positive
    [Tags]    window
    ${window}=    Create Orbital Window    start=-1h    end=+1h
    Should Be True    ${window}[start_time] < ${window}[end_time]

Window With Past Start
    [Tags]    window
    ${window}=    Create Orbital Window    start=-2h    end=-1h
    Should Be Valid Window    ${window}

Window With Future Start
    [Tags]    window
    ${window}=    Create Orbital Window    start=+1h    end=+2h
    Should Be Valid Window    ${window}

Window Generates UUID
    [Tags]    window
    ${window}=    Create Orbital Window    start=-1h    end=+1h
    ${uuid}=    Set Variable    ${window}[id]
    Should Match Regex    ${uuid}    ^[a-f0-9-]{36}$

Window Reports Type
    [Tags]    window
    ${window}=    Create Orbital Window    start=-1h    end=+1h    window_type=Standard
    Should Be Equal    ${window}[window_type]    Standard

Create Window With Zero Duration
    [Tags]    window    negative
    Run Keyword And Expect Error    *400*    Create Orbital Window    start=now    end=now

Create Window With Negative Duration
    [Tags]    window    negative
    Run Keyword And Expect Error    *400*    Create Orbital Window    start=+1h    end=-1h

Multiple Windows Can Be Created
    [Tags]    window
    ${w1}=    Create Orbital Window    start=-1h    end=+1h
    ${w2}=    Create Orbital Window    start=-2h    end=+2h
    Should Not Be Equal    ${w1}[id]    ${w2}[id]

Scheduled Window Generates Events
    [Tags]    window
    ${window}=    Create Orbital Window    start=-5m    end=+5m    window_type=Standard
    Sleep    2s

Create Window Sets Correct Times
    [Tags]    window
    ${window}=    Create Orbital Window    start=-1h    end=+1h
    Should Not Be Empty    ${window}[start_time]
    Should Not Be Empty    ${window}[end_time]
