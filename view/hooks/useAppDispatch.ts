import { TypedUseSelectorHook, useDispatch, useSelector } from "react-redux"
import { RootState, StateDispatch } from "../redux/store"

const useAppDispatch = () => useDispatch<StateDispatch>();

export default useAppDispatch;