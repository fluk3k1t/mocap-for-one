port module Main exposing (..)

import Browser
import Html exposing (Html, button, div, text)
import Html.Events exposing (onClick)
import Material.Card exposing (button)
import Material.Tab as Tab
import Material.TabBar as TabBar


type alias Model =
    { selectedTab : Int
    , cameras : List String
    , message : String
    }


type Msg
    = TabClicked Int
    | Recv String
    | Send String
    | Call String (String -> Msg)


port sendMessage : String -> Cmd msg


port messageReceiver : (String -> msg) -> Sub msg


main =
    Browser.element { init = init, update = update, view = view, subscriptions = subscriptions }


init : () -> ( Model, Cmd Msg )
init () =
    ( { selectedTab = 0
      , cameras = [ "Camera 1" ]
      , message = ""
      }
    , Cmd.none
    )


update : Msg -> Model -> ( Model, Cmd Msg )
update msg model =
    case msg of
        TabClicked index ->
            ( { model | selectedTab = index }, Cmd.none )

        Send sendMsg ->
            ( model, sendMessage sendMsg )

        Recv recvMsg ->
            ( { model | message = recvMsg }, Cmd.none )

        Call callMsg k ->
            ( model, Cmd.none )


subscriptions : Model -> Sub Msg
subscriptions model =
    messageReceiver Recv


view : Model -> Html Msg
view model =
    div []
        [ TabBar.tabBar TabBar.config
            (Tab.tab
                (Tab.config
                    |> Tab.setActive (model.selectedTab == 0)
                    |> Tab.setOnClick (TabClicked 0)
                )
                { label = Maybe.withDefault "" (List.head model.cameras), icon = Nothing }
            )
            []
        , Html.button [ onClick <| Send "Hello" ] [ text "Send" ]
        , text model.message

        -- (List.indexedMap
        --     (\index camera ->
        --         Tab.tab
        --             (Tab.config
        --                 |> Tab.setOnClick (TabClicked (index + 1))
        --                 |> Tab.setActive (model.selectedTab == (index + 1))
        --             )
        --             { label = camera, icon = Nothing }
        --     )
        --     (List.drop 1 model.cameras)
        -- )
        ]
