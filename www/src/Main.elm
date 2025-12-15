port module Main exposing (..)

import Browser
import Html exposing (Html, button, div, text)
import Html.Events exposing (onClick)
import Json.Decode
import Material.Card exposing (button)
import Material.Tab as Tab
import Material.TabBar as TabBar
import Random
import Task exposing (Task)
import TaskPort
import Time
import UUID exposing (UUID)


newUuid _ =
    Random.step UUID.generator (Random.initialSeed 12345)
        |> Tuple.first
        |> UUID.toRepresentation UUID.Urn


type alias Model =
    { selectedTab : Int
    , cameras : List String
    , message : String
    , called : List ( String, String -> Msg )
    }


type Msg
    = Recv String
    | Send String
    | Call String (String -> Msg)
    | NewUUID UUID
    | TestFunction (TaskPort.Result String)


port sendMessage : String -> Cmd msg


port messageReceiver : (String -> msg) -> Sub msg


testFunction =
    TaskPort.callNoArgs
        { function = "functionName"
        , valueDecoder = Json.Decode.string
        }


main =
    Browser.element { init = init, update = update, view = view, subscriptions = subscriptions }


init : () -> ( Model, Cmd Msg )
init () =
    ( { selectedTab = 0
      , cameras = [ "Camera 1" ]
      , message = ""
      , called = []
      }
    , Task.attempt TestFunction testFunction
    )


update : Msg -> Model -> ( Model, Cmd Msg )
update msg model =
    case msg of
        Send sendMsg ->
            ( model, sendMessage sendMsg )

        Recv recvMsg ->
            ( { model | message = recvMsg }, Cmd.none )

        Call callMsg k ->
            ( model, Cmd.none )

        NewUUID uuid ->
            ( model, Cmd.none )

        TestFunction result ->
            case result of
                Ok value ->
                    ( { model | message = value }, Cmd.none )

                Err error ->
                    ( { model | message = TaskPort.errorToString error }, Cmd.none )


subscriptions : Model -> Sub Msg
subscriptions model =
    messageReceiver Recv


view : Model -> Html Msg
view model =
    div []
        [ Html.button [ onClick <| Send "Hello" ] [ text "Send" ]
        , Html.button [] [ text (newUuid ()) ]
        , Html.button [] [ text (newUuid ()) ]
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
