import { get_jwt } from "../helpers/cookies.js";

export function with_jwt_server_side() {
    return async (context) => {
        const { req } = context;
        const jwt = get_jwt({req});
        return {props: {
            jwt,
        }};
    }
}
