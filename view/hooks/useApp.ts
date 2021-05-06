import {TypedUseSelectorHook, useDispatch, useSelector} from "react-redux"
import { RootState, StateDispatch } from "../redux/store"

export const useAppDispatch = () => useDispatch<StateDispatch>();
export const useAppSelector: TypedUseSelectorHook<RootState> = useSelector;