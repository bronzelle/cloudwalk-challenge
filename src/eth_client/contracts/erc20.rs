use alloy_sol_types::sol;

sol! {
    /// The ERC-20 token standard interface.
    #[sol(rpc)]
    interface IERC20 {
        // Events
        event Transfer(address indexed from, address indexed to, uint256 value);
        event Approval(address indexed owner, address indexed spender, uint256 value);

        // Functions
        function name() external view returns (string memory);
        function symbol() external view returns (string memory);
        function decimals() external view returns (uint8);
        function totalSupply() external view returns (uint256);
        function balanceOf(address account) external view returns (uint256);
        function transfer(address to, uint256 value) external returns (bool);
    }
}
